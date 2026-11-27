impl MusicApp {
    fn lyrics_ui(&mut self, ui: &mut egui::Ui) {
        let Some(track) = self
            .selected
            .clone()
            .or_else(|| self.player.current().cloned())
        else {
            ui.label("Chọn bài hát để sửa lời.");
            return;
        };
        if self.editor_track != track.path {
            if !self.switch_editor(&track) {
                return;
            }
            self.request_lyrics(&track);
            self.sync_editor();
        }
        ui.label(RichText::new(&track.title).strong());
        ui.label(&track.artist);
        if let Some(result) = self.lyric_cache.get(&track.path) {
            match result {
                Err(e) => {
                    ui.colored_label(ui.visuals().error_fg_color, e);
                }
                Ok(Some(r)) => {
                    ui.label(format!(
                        "Nguồn: {} · Mốc: {}",
                        r.source,
                        if r.draft_only {
                            "Bản nháp chưa xác nhận"
                        } else {
                            &r.timing
                        }
                    ));
                }
                _ => {}
            }
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("Tra cứu lời có sẵn").clicked() {
                self.lyric_preview(track.clone(), true, false);
            }
            if ui.button("Tạo / viết lại từ âm thanh").clicked() {
                self.lyric_preview(track.clone(), false, true);
            }
            if ui.button("AI rà soát chính tả").clicked() {
                let app = self.services.clone();
                let content = self.editor.clone();
                let key = track.path.clone();
                let rev = self.editor_revision;
                self.task(async move {
                    let reviewed = lyrics_jobs::lyrics_review_text(app, content).await?;
                    Ok(Message::Reviewed(key, rev, reviewed.content))
                });
            }
            if ui.button("Căn theo giọng hát").clicked() {
                let app = self.services.clone();
                let content = self.editor.clone();
                let key = track.path.clone();
                let rev = self.editor_revision;
                let duration = track.duration;
                let q: LyricsQuery = track.clone().into();
                self.task(async move {
                    let transcript = lyrics_jobs::lyrics_transcribe(app, q).await?;
                    if transcript.draft_only {
                        return Err("Kết quả nhận diện còn chỗ chưa rõ. Không tự gán mốc.".into());
                    }
                    let aligned = lrc::align(&content, &transcript.content, duration)?;
                    Ok(Message::Aligned(key, rev, aligned))
                });
            }
            if ui.button("Cài đặt AI").clicked() {
                self.page = Page::Settings;
            }
        });
        let mut offset = lrc::offset(&self.editor);
        ui.horizontal(|ui| {
            ui.label("Bù thời gian (ms)");
            if ui
                .add(
                    egui::DragValue::new(&mut offset)
                        .range(-10000..=10000)
                        .speed(10.),
                )
                .changed()
            {
                match lrc::set_offset(&self.editor, offset) {
                    Ok(s) => {
                        self.editor = s;
                        self.editor_dirty = true;
                        self.editor_revision += 1;
                    }
                    Err(e) => self.fail(e),
                }
            }
            ui.label("Số dương hiển thị lời sớm hơn.");
        });
        if let Some(preview) = self.preview.clone() {
            egui::CollapsingHeader::new("Bản xem trước — chưa thay đổi lời đã lưu")
                .default_open(true)
                .show(ui, |ui| {
                    if preview.draft_only {
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            "Bản nháp có chỗ chưa nghe rõ. Chưa có mốc xác nhận.",
                        );
                    }
                    let mut text = preview.content.clone();
                    egui::ScrollArea::vertical()
                        .max_height(150.)
                        .id_salt("preview")
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut text)
                                    .interactive(false)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Dùng để chỉnh sửa").clicked() {
                            self.editor = preview.content.clone();
                            self.editor_dirty = true;
                            self.editor_revision += 1;
                            self.preview = None;
                        }
                        if ui.button("Bỏ bản xem trước").clicked() {
                            self.preview = None;
                        }
                    });
                });
        }
        egui::CollapsingHeader::new("Sửa chính tả, giữ nguyên mốc từng chữ").show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.spelling)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3),
            );
            if ui.button("Áp dụng chính tả").clicked() {
                match lrc::correct_text(&self.editor, &self.spelling) {
                    Ok(s) => {
                        self.editor = s;
                        self.editor_dirty = true;
                        self.editor_revision += 1;
                    }
                    Err(e) => self.fail(e),
                }
            }
        });
        ui.label("Nội dung LRC / lời bài hát");
        let available = (ui.available_height() - 90.).max(100.);
        egui::ScrollArea::vertical()
            .id_salt("lrc-editor")
            .max_height(available)
            .show(ui, |ui| {
                if ui
                    .add(
                        egui::TextEdit::multiline(&mut self.editor)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(10),
                    )
                    .changed()
                {
                    self.editor_dirty = true;
                    self.editor_revision += 1;
                }
            });
        if self.editor.contains(lyrics::UNRESOLVED_MARKER) {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                "Còn đoạn chưa nghe rõ. Hãy sửa trước khi lưu bản hoàn chỉnh.",
            );
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    !self.editor.trim().is_empty()
                        && !self.editor.contains(lyrics::UNRESOLVED_MARKER),
                    egui::Button::new("Lưu lời đã kiểm tra"),
                )
                .clicked()
            {
                let app = self.services.clone();
                let q: LyricsQuery = track.clone().into();
                let content = self.editor.clone();
                let revision = self.editor_revision;
                self.task(async move {
                    tokio::task::spawn_blocking(move || {
                        lyrics_jobs::valid_query(&q)?;
                        let s = app.state::<lyrics_jobs::LyricsService>();
                        let _gate = s
                            .gate
                            .try_lock()
                            .map_err(|_| "Đang có tác vụ lời khác. Chờ xong rồi lưu.")?;
                        lyrics::save_manual(&s.cache_dir(), &q, &content)?;
                        let _ = app.emit("lyrics-updated", &q.path);
                        Ok(Message::Saved(q.path, revision))
                    })
                    .await
                    .map_err(|e| e.to_string())?
                });
            }
            if ui.button("Xuất LRC").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .set_file_name(format!("{}.lrc", safe_name(&track.title)))
                    .save_file()
            {
                if let Err(e) = atomic_text(&path, &self.editor) {
                    self.fail(e);
                } else {
                    self.notice("Đã xuất LRC.");
                }
            }
            if ui.button("Nhập LRC").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Lời bài hát", &["lrc", "txt"])
                    .pick_file()
            {
                match lyrics::read_limited(&path, lyrics::MAX_LYRICS)
                    .and_then(|s| String::from_utf8(s).map_err(|e| e.to_string()))
                {
                    Ok(s) => {
                        self.editor = s;
                        self.editor_dirty = true;
                        self.editor_revision += 1;
                    }
                    Err(e) => self.fail(e),
                }
            }
            if ui.button("Xóa lời trong app").clicked() {
                self.confirm = Some(Confirmation::Lyrics(track.clone()));
            }
        });
    }
    fn lyric_preview(&mut self, track: Track, refresh: bool, rewrite: bool) {
        let app = self.services.clone();
        let key = track.path.clone();
        let rev = self.editor_revision;
        self.task(async move {
            let result =
                lyrics_jobs::lyrics_generate(app, track.into(), Some(refresh), Some(rewrite))
                    .await?;
            Ok(Message::Preview(key, rev, result))
        });
    }
    fn vinyl(&mut self, ui: &mut egui::Ui, size: f32) {
        let size = size.min(ui.available_width()).max(40.);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
        let painter = ui.painter_at(rect);
        let center = rect.center();
        let radius = size * 0.44;
        let accent = Color32::from_rgb(
            self.settings.vinyl_color[0],
            self.settings.vinyl_color[1],
            self.settings.vinyl_color[2],
        );
        painter.circle_filled(
            center + Vec2::new(0., 5.),
            radius + 3.,
            Color32::from_black_alpha(55),
        );
        painter.circle_filled(center, radius, Color32::from_rgb(20, 20, 27));
        let rotation = if self.player.phase == Phase::Playing {
            ui.input(|i| i.time) as f32 * self.settings.rpm / 60. * std::f32::consts::TAU
        } else {
            0.
        };
        let cover = self.player.current().and_then(|t| t.cover_path.clone());
        if self.settings.skin == "album"
            && let Some(cover) = cover
        {
            let image = egui::Image::new(uri(&cover)).rotate(rotation, Vec2::splat(0.5));
            image.paint_at(
                ui,
                egui::Rect::from_center_size(center, Vec2::splat(radius * 1.25)),
            );
        }
        for i in 0..30 {
            let r = radius * (0.35 + i as f32 * 0.020);
            painter.circle_stroke(
                center,
                r,
                Stroke::new(
                    0.6_f32,
                    Color32::from_white_alpha(if i % 3 == 0 { 30 } else { 12 }),
                ),
            );
        }
        if self.settings.skin == "marble" || self.settings.skin == "custom" {
            for i in 0..10 {
                let angle = rotation + i as f32 * 0.62;
                let a = center + Vec2::angled(angle) * (radius * 0.8);
                let b = center + Vec2::angled(angle + 0.65) * (radius * 0.45);
                painter.line_segment([a, b], Stroke::new(1.5_f32, accent.gamma_multiply(0.25)));
            }
        }
        let label = match self.settings.skin.as_str() {
            "gold" => Color32::from_rgb(211, 168, 76),
            "cyberpunk" => Color32::from_rgb(230, 67, 158),
            "black" => Color32::from_gray(65),
            _ => accent,
        };
        painter.circle_filled(center, radius * 0.30, label);
        painter.circle_stroke(
            center,
            radius * 0.28,
            Stroke::new(1.0_f32, Color32::from_white_alpha(75)),
        );
        painter.circle_filled(center, radius * 0.065, Color32::from_rgb(20, 20, 27));
        painter.circle_filled(
            center
                + Vec2::new(
                    radius * 0.18 * rotation.cos(),
                    radius * 0.18 * rotation.sin(),
                ),
            2.,
            Color32::from_white_alpha(180),
        );
        if self.settings.tonearm {
            let pivot = rect.right_top() + Vec2::new(-size * 0.08, size * 0.12);
            let end = if self.player.phase == Phase::Playing {
                center + Vec2::new(radius * 0.55, radius * 0.2)
            } else {
                pivot + Vec2::new(-size * 0.03, size * 0.65)
            };
            painter.circle_filled(pivot, 7., Color32::from_gray(100));
            painter.line_segment([pivot, end], Stroke::new(5.0_f32, Color32::from_gray(175)));
            painter.circle_filled(end, 4., Color32::from_gray(210));
        }
        if response.clicked() {
            self.toggle();
        }
        response.on_hover_text("Bấm đĩa để phát hoặc tạm dừng");
        if self.player.phase == Phase::Playing {
            ui.ctx().request_repaint_after(Duration::from_millis(33));
        }
    }
    fn karaoke_options(&mut self, ui: &mut egui::Ui) {
        if ui
            .button(if self.controls {
                "Ẩn điều khiển"
            } else {
                "Hiện điều khiển"
            })
            .clicked()
        {
            self.controls = !self.controls;
        }
        if ui.button("Kết thúc karaoke").clicked() {
            self.player.karaoke = false;
            self.player.reorder();
            self.karaoke_window = false;
            self.mini = false;
            self.page = Page::Library;
        }
        if ui.button("Gọi bài").clicked() {
            self.desk_open = !self.desk_open;
        }
        egui::ComboBox::from_id_salt("lyrics-layout")
            .selected_text(match self.settings.layout.as_str() {
                "stage" => "Sân khấu",
                "full" => "Toàn bộ lời",
                _ => "Đĩa than",
            })
            .show_ui(ui, |ui| {
                for (value, label) in [
                    ("vinyl", "Đĩa than"),
                    ("stage", "Sân khấu"),
                    ("full", "Toàn bộ lời"),
                ] {
                    if ui
                        .selectable_value(&mut self.settings.layout, value.into(), label)
                        .changed()
                    {
                        self.save_settings();
                    }
                }
            });
        if ui
            .button(if self.karaoke_window {
                "Về cửa sổ chính"
            } else {
                "Cửa sổ riêng"
            })
            .clicked()
        {
            self.karaoke_window = !self.karaoke_window;
        }
        if ui.button("Toàn màn hình").clicked() {
            self.karaoke_fullscreen = !self.karaoke_fullscreen;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.karaoke_fullscreen));
        }
        if ui
            .button(if self.mini {
                "Mở rộng"
            } else {
                "Thu nhỏ"
            })
            .clicked()
        {
            self.mini = !self.mini;
            if self.mini {
                self.page = Page::Library;
            }
        }
    }
    fn karaoke_ui(&mut self, ui: &mut egui::Ui) {
        let compact = ui.available_height() < 400.;
        ui.horizontal_wrapped(|ui| {
            if compact {
                if ui.button("Gọi bài").clicked() {
                    self.desk_open = !self.desk_open;
                }
                ui.menu_button("Tùy chọn Studio", |ui| self.karaoke_options(ui));
            } else {
                self.karaoke_options(ui);
            }
        });
        let Some(track) = self.player.current().cloned() else {
            ui.vertical_centered(|ui| {
                ui.add_space(40.);
                ui.heading("Bài đầu tiên, bạn chọn nhé.");
                ui.label("Mở Gọi bài để chọn từ thư viện hoặc tìm trực tuyến.");
                if ui.button("Mở gọi bài").clicked() {
                    self.desk_open = true;
                }
            });
            if self.desk_open {
                self.karaoke_desk(ui);
            }
            return;
        };
        self.request_lyrics(&track);
        if self.controls {
            ui.label(RichText::new(&track.title).size(21.).strong());
            ui.label(&track.artist);
        }
        if self.desk_open {
            self.karaoke_desk(ui);
            return;
        }
        let lyrics = self.lyric_cache.get(&track.path).cloned();
        let local_transport = self.controls && (!compact || self.karaoke_window);
        let area = (ui.available_height() - if local_transport { 100. } else { 0. }).max(40.);
        if self.settings.layout == "vinyl" && ui.available_width() > 720. {
            ui.horizontal(|ui| {
                self.vinyl(ui, (area * 0.85).min(ui.available_width() * 0.42));
                ui.allocate_ui(Vec2::new(ui.available_width(), area), |ui| {
                    self.singing_lyrics(ui, lyrics.clone(), area);
                });
            });
        } else {
            self.singing_lyrics(ui, lyrics, area);
        }
        if local_transport {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Tìm bản beat").clicked() {
                    self.online_query = format!("{} {} karaoke beat", track.title, track.artist);
                    self.source = "youtube".into();
                    self.desk_tab = 1;
                    self.desk_open = true;
                    self.search(true);
                }
                if ui.button("Bản có giọng").clicked() {
                    self.online_query = format!("{} {} official audio", track.title, track.artist);
                    self.source = "youtube".into();
                    self.desk_tab = 1;
                    self.desk_open = true;
                    self.search(true);
                }
                if ui.button("Sửa lời").clicked() {
                    self.open_track(track.clone(), Page::Karaoke);
                    self.desk_tab = 2;
                    self.desk_open = true;
                }
                if ui
                    .button(if self.player.phase == Phase::Playing {
                        "Tạm dừng"
                    } else {
                        "Phát"
                    })
                    .clicked()
                {
                    self.toggle();
                }
                if ui.button("Bỏ qua").clicked() {
                    self.next(false, false);
                }
            });
            let mut position = self.audio.position();
            ui.horizontal(|ui| {
                ui.monospace(time(position));
                if ui
                    .add(
                        egui::Slider::new(&mut position, 0.0..=self.duration().max(1.))
                            .show_value(false),
                    )
                    .changed()
                    && let Err(e) = self.audio.seek(position)
                {
                    self.fail(e);
                }
                ui.monospace(time(self.duration()));
            });
        }
    }
    fn singing_lyrics(
        &mut self,
        ui: &mut egui::Ui,
        result: Option<Result<Option<LyricsResult>, String>>,
        height: f32,
    ) {
        let position = self.audio.position();
        let accent = self.accent(ui.ctx());
        egui::ScrollArea::vertical()
            .id_salt("singing")
            .max_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| match result {
                None => {
                    ui.spinner();
                    ui.label("Đang tải lời bài hát…");
                }
                Some(Err(error)) => {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                    if ui.button("Tải lại lời").clicked()
                        && let Some(t) = self.player.current().cloned()
                    {
                        self.lyric_cache.remove(&t.path);
                        self.request_lyrics(&t);
                    }
                }
                Some(Ok(None)) => {
                    ui.add_space(height * 0.2);
                    ui.heading("Hát theo giai điệu của bạn");
                    ui.label("Bản này chưa có lời. Mở Sửa lời để thêm hoặc tìm lời.");
                }
                Some(Ok(Some(result))) => {
                    if result.draft_only {
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            "Bản nháp chưa xác nhận — chưa căn mốc từng chữ.",
                        );
                        ui.label(result.content);
                        return;
                    }
                    let parsed = lrc::parse(&result.content);
                    let active = lrc::active(&parsed.lines, position);
                    if !parsed.synced || self.settings.layout == "full" {
                        for (i, line) in parsed.lines.iter().enumerate() {
                            let selected = active == Some(i);
                            let response = ui.add(
                                egui::Label::new(
                                    RichText::new(&line.text)
                                        .size(if selected {
                                            self.settings.font_size
                                        } else {
                                            self.settings.font_size * 0.72
                                        })
                                        .color(if selected {
                                            accent
                                        } else {
                                            ui.visuals().text_color()
                                        }),
                                )
                                .wrap()
                                .sense(Sense::click()),
                            );
                            if selected && self.player.phase == Phase::Playing {
                                response.scroll_to_me(Some(Align::Center));
                            }
                            if response.clicked()
                                && line.time >= 0.
                                && let Err(e) = self.audio.seek(line.time)
                            {
                                self.fail(e);
                            }
                            ui.add_space(12.);
                        }
                    } else {
                        ui.add_space((height * 0.10).min(40.));
                        ui.vertical_centered(|ui| {
                            if let Some(i) = active {
                                if i > 0 {
                                    ui.label(
                                        RichText::new(&parsed.lines[i - 1].text)
                                            .size(self.settings.font_size * 0.55)
                                            .color(
                                                ui.visuals()
                                                    .text_color()
                                                    .gamma_multiply(self.settings.dim_inactive),
                                            ),
                                    );
                                }
                                ui.add_space(24.);
                                let line = &parsed.lines[i];
                                if line.word_sync {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 6.;
                                        for word in &line.words {
                                            let fill = ((position - word.start)
                                                / (word.end - word.start).max(0.01))
                                            .clamp(0., 1.)
                                                as f32;
                                            let color = if fill > 0. {
                                                accent
                                            } else {
                                                ui.visuals().text_color().gamma_multiply(0.75)
                                            };
                                            let response = ui.label(
                                                RichText::new(&word.text)
                                                    .size(self.settings.font_size)
                                                    .strong()
                                                    .color(color),
                                            );
                                            if fill > 0. && fill < 1. {
                                                let r = response.rect;
                                                ui.painter().line_segment(
                                                    [
                                                        r.left_bottom(),
                                                        r.left_bottom()
                                                            + Vec2::new(r.width() * fill, 0.),
                                                    ],
                                                    Stroke::new(2.0_f32, accent),
                                                );
                                            }
                                        }
                                    });
                                } else {
                                    ui.label(
                                        RichText::new(&line.text)
                                            .size(self.settings.font_size)
                                            .strong()
                                            .color(accent),
                                    );
                                    ui.label(RichText::new("Mốc theo câu").small());
                                }
                                ui.add_space(24.);
                                if let Some(next) = parsed.lines.get(i + 1) {
                                    ui.label(
                                        RichText::new(&next.text)
                                            .size(self.settings.font_size * 0.65),
                                    );
                                } else {
                                    ui.label("Cảm ơn bạn đã hát cùng Trantramy.");
                                }
                            } else {
                                ui.heading("Nhạc dạo");
                                if let Some(first) = parsed.lines.first() {
                                    ui.label(&first.text);
                                }
                            }
                        });
                    }
                }
            });
    }
    fn karaoke_desk(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (i, label) in ["Thư viện", "Trực tuyến", "Sửa lời", "Chờ hát"]
                .iter()
                .enumerate()
            {
                ui.selectable_value(&mut self.desk_tab, i, *label);
            }
            if ui.button("Về sân khấu").clicked() {
                self.desk_open = false;
            }
        });
        ui.separator();
        match self.desk_tab {
            0 => {
                ui.add(
                    egui::TextEdit::singleline(&mut self.query).hint_text("Tìm bài trong thư viện"),
                );
                let tracks = self.filtered_tracks();
                self.track_list(ui, &tracks, false);
            }
            1 => {
                ui.horizontal_wrapped(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.online_query).desired_width(240.));
                    if ui.button("Tìm").clicked() {
                        self.search(self.replace_source);
                    }
                });
                if self.replace_source {
                    ui.label("Chọn nguồn thay cho bài đang hát. Bản mới phát từ đầu.");
                    if ui.button("Hủy đổi bản").clicked() {
                        self.replace_source = false;
                    }
                }
                let tracks = self.online.clone();
                egui::ScrollArea::vertical()
                    .id_salt("karaoke-online")
                    .show(ui, |ui| {
                        for t in tracks {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(&t.title);
                                ui.label(&t.artist);
                                if self.replace_source {
                                    if ui.button("Dùng bản này").clicked() {
                                        self.player.replace(t.clone());
                                        self.load_current();
                                        self.replace_source = false;
                                        self.desk_open = false;
                                    }
                                } else if ui.button("Gọi bài").clicked() {
                                    self.enqueue(t.clone());
                                }
                            });
                            ui.separator();
                        }
                    });
            }
            2 => {
                if let Some(t) = self.player.current().cloned()
                    && self.selected.as_ref().is_none_or(|s| s.path != t.path)
                {
                    self.open_track(t, Page::Karaoke);
                }
                self.lyrics_ui(ui);
            }
            _ => self.queue_ui(ui),
        }
    }
}
