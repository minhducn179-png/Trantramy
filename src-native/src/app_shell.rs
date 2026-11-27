impl MusicApp {
    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().id_salt("settings").show(ui,|ui|{
            ui.heading("Giao diện và Studio");let mut changed=false;
            egui::ComboBox::from_id_salt("palette").selected_text(self.palettes.iter().find(|p|p.id==self.settings.palette).map_or(self.settings.palette.clone(),|p|format!("{} / {}",p.family,p.name))).show_ui(ui,|ui|{for p in &self.palettes{changed|=ui.selectable_value(&mut self.settings.palette,p.id.clone(),format!("{} / {}",p.family,p.name)).changed();}});
            ui.horizontal_wrapped(|ui|{if ui.button("Chọn hình nền").clicked()&& let Some(path)=rfd::FileDialog::new().add_filter("Ảnh",&["jpg","png","jpeg","webp"]).pick_file(){match copy_image(&path,&self.services.path().data.join("backgrounds")){Ok(p)=>{self.settings.wallpaper=Some(p);changed=true;},Err(e)=>self.fail(e)}}
if ui.button("Bỏ hình nền").clicked(){self.settings.wallpaper=None;changed=true;}});
            changed|=ui.add(egui::Slider::new(&mut self.settings.wallpaper_opacity,0.0..=0.6).text("Độ rõ hình nền")).changed();
            changed|=ui.add(egui::Slider::new(&mut self.settings.rpm,20.0..=78.0).text("Tốc độ đĩa (RPM)")).changed();changed|=ui.checkbox(&mut self.settings.tonearm,"Hiện cần đĩa").changed();
            ui.horizontal_wrapped(|ui|{ui.label("Mặt đĩa");for(id,label)in[("marble","Marble"),("album","Ảnh bìa"),("black","Đen"),("cyberpunk","Cyberpunk"),("gold","Vàng"),("custom","Tùy chọn")]{changed|=ui.selectable_value(&mut self.settings.skin,id.into(),label).changed();}});
            changed|=ui.color_edit_button_srgb(&mut self.settings.vinyl_color).changed();
            changed|=ui.add(egui::Slider::new(&mut self.settings.font_size,16.0..=72.0).text("Cỡ chữ karaoke")).changed();changed|=ui.add(egui::Slider::new(&mut self.settings.dim_inactive,0.2..=1.0).text("Độ sáng câu trước")).changed();
            changed|=ui.checkbox(&mut self.settings.island,"Dynamic Island — cửa sổ nhỏ luôn ở trên").changed();changed|=ui.checkbox(&mut self.settings.stealth,"Bảo vệ cửa sổ khỏi chụp màn hình (nếu hệ điều hành hỗ trợ)").changed();changed|=ui.checkbox(&mut self.settings.auto_related,"Tìm gợi ý khi hết hàng chờ").changed();if changed{self.save_settings();}
            ui.add_space(18.);ui.separator();ui.heading("Âm thanh · EQ 10 dải");
            let mut eq=self.audio.eq.lock().map(|e|e.clone()).unwrap_or_default();let before=eq.clone();ui.checkbox(&mut eq.enabled,"Bật EQ");ui.checkbox(&mut eq.limiter,"Giới hạn biên độ để giảm clipping");ui.add(egui::Slider::new(&mut eq.preamp,-12.0..=12.0).text("Preamp (dB)"));
            ui.horizontal_wrapped(|ui|{for(label,gains)in[("Phẳng",[0.;10]),("Bass",[6.,5.,3.5,1.5,0.,0.,0.,0.,0.,0.]),("Treble",[0.,0.,0.,0.,0.,1.,2.5,4.,5.5,6.]),("Vocal",[-2.,-1.,0.5,2.,4.,4.,3.,1.5,0.5,-1.]),("Pop",[-1.,1.5,3.,3.5,2.5,0.,-1.,1.5,3.,3.5]),("Rock",[4.5,3.5,1.5,-1.,-1.5,0.,1.5,3.,4.,4.5]),("EDM",[6.,4.5,2.,0.,-1.,1.,2.5,4.,5.,4.5]),("Acoustic",[3.,2.5,1.5,1.,2.,2.,2.5,3.,3.5,2.5]),("Cổ điển",[4.,3.,2.,1.5,-0.5,-0.5,0.,1.5,2.5,3.]),("Jazz",[3.5,2.5,1.,1.5,-1.,-1.,0.,1.,2.5,3.])]{if ui.button(label).clicked(){eq.gains=gains;}}});
            egui::Grid::new("eq-bands").num_columns(2).show(ui,|ui|{for(i,freq)in audio::FREQUENCIES.iter().enumerate(){ui.label(format!("{freq:.0} Hz"));ui.add(egui::Slider::new(&mut eq.gains[i],-12.0..=12.0).suffix(" dB"));ui.end_row();}});if eq!=before{if let Ok(mut e)=self.audio.eq.lock(){*e=eq;}self.save_settings();}
            ui.add_space(18.);ui.separator();ui.heading("Lời bài hát và AI");
            egui::ComboBox::from_id_salt("provider").selected_text(&self.ai.provider).show_ui(ui,|ui|{for(id,label)in[("offline","Whisper offline"),("gemini","Gemini"),("openai","OpenAI"),("groq","Groq")]{ui.selectable_value(&mut self.ai.provider,id.into(),label);}});
            ui.label("Gemini nhiều bước cần quyền gửi âm thanh. Whisper offline chạy trên máy.");ui.checkbox(&mut self.ai.cloud_consent,"Cho phép gửi âm thanh tới nhà cung cấp AI đã chọn");ui.checkbox(&mut self.ai.text_consent,"Cho phép gửi lời bài hát để AI rà soát / đối chiếu");
            ui.checkbox(&mut self.ai.automatic,"Tự tìm / tạo lời khi mở bài");ui.checkbox(&mut self.ai.network_lookup,"Tra cứu lời trên mạng");ui.checkbox(&mut self.ai.lyricsplus_lookup,"Cho phép nguồn LyricsPlus");ui.checkbox(&mut self.ai.audio_pipeline,"Quy trình Gemini nhiều bước đối chiếu âm thanh");ui.checkbox(&mut self.ai.context_rewrite,"Đối chiếu ngữ cảnh để sửa lời");ui.checkbox(&mut self.ai.auto_review,"Rà soát sau nhận diện");ui.checkbox(&mut self.ai.offline_fallback,"Dự phòng Whisper offline");
            egui::ComboBox::from_id_salt("language").selected_text(&self.ai.language).show_ui(ui,|ui|{for(id,label)in[("auto","Tự nhận ngôn ngữ"),("vi","Tiếng Việt"),("en","English")]{ui.selectable_value(&mut self.ai.language,id.into(),label);}});
            egui::ComboBox::from_id_salt("whisper-model").selected_text(&self.ai.model).show_ui(ui,|ui|{for m in["base","small"]{ui.selectable_value(&mut self.ai.model,m.into(),m);}});
            egui::ComboBox::from_id_salt("gemini-model").selected_text(&self.ai.gemini_model).show_ui(ui,|ui|{for m in["gemini-2.5-flash","gemini-2.5-flash-lite","gemini-3-flash-preview"]{ui.selectable_value(&mut self.ai.gemini_model,m.into(),m);}});
            ui.add(egui::Slider::new(&mut self.ai.max_audio_seconds,30..=720).text("Giây âm thanh / lượt"));ui.add(egui::Slider::new(&mut self.ai.daily_cloud_seconds,0..=36000).text("Giới hạn giây cloud / ngày"));ui.add(egui::Slider::new(&mut self.ai.batch_limit,1..=500).text("Số bài / lượt thư viện"));
            if self.ai.provider!="offline"{ui.label("API key — lưu trong kho khóa hệ điều hành");ui.add(egui::TextEdit::singleline(&mut self.api_key).password(true).hint_text("Để trống để giữ khóa hiện tại"));}
            ui.horizontal_wrapped(|ui|{
                if ui.button("Lưu cấu hình và quyền đã chọn").clicked(){let app=self.services.clone();let config=self.ai.clone();let key=if self.api_key.is_empty(){None}else{Some(std::mem::take(&mut self.api_key))};self.task(async move{lyrics_jobs::lyrics_save_settings(app,config,key).await?;Ok(Message::Done("Đã lưu cấu hình và quyền dữ liệu bạn đã chọn.".into()))});}
                if ui.button("Cập nhật trạng thái AI").clicked(){self.refresh_ai();}
if ui.button("Mở thư mục lời").clicked(){match lyrics_jobs::open_lyrics_folder(self.services.clone()){Ok(s)=>self.notice(s),Err(e)=>self.fail(e)}}
            });
            if let Some(view)=&self.settings_view{ui.label(format!("Cloud hôm nay: {} giây · {} bản lời lưu · khóa đã có: {}",view.cloud_seconds_used,view.lyrics_count,view.keys.join(", ")));for model in &view.models{ui.label(serde_json::to_string(model).unwrap_or_default());}}
            ui.horizontal_wrapped(|ui|{
                if ui.button("Tải model offline").clicked(){let app=self.services.clone();let model=self.ai.model.clone();self.task(async move{lyrics_jobs::lyrics_download_model(app,model).await?;Ok(Message::Done("Đã tải model offline.".into()))});}
                if ui.button("Hủy tải model").clicked(){lyrics_jobs::lyrics_cancel_download(self.services.clone());}
if ui.button("Tạo lời cho thư viện").clicked(){self.start_lyrics_queue(false);}
if ui.button("Tiếp tục hàng đợi").clicked(){self.start_lyrics_queue(true);}
if ui.button("Tạm dừng").clicked()&& let Err(e)=lyrics_jobs::lyrics_pause(self.services.clone()){self.fail(e);}
            });
            if let Ok(queue)=lyrics_jobs::lyrics_queue(self.services.clone()){if !queue.error.is_empty(){ui.colored_label(ui.visuals().error_fg_color,queue.error);}for job in queue.jobs.iter().take(100){ui.label(format!("{} · {} · {}",job.track.title,job.state,job.message));}}
            ui.add_space(16.);ui.label("Thư viện SQLite, bộ nhớ lời và khóa AI dùng cùng vị trí dữ liệu của bản trước. Tùy chỉnh giao diện native lưu riêng.");
        });
    }
    fn refresh_ai(&mut self) {
        let app = self.services.clone();
        self.task(async move { Ok(Message::Settings(lyrics_jobs::lyrics_settings(app).await?)) });
    }
    fn start_lyrics_queue(&mut self, resume: bool) {
        let app = self.services.clone();
        self.task(async move {
            lyrics_jobs::lyrics_start_library(app, resume).await?;
            Ok(Message::Done("Đã bắt đầu hàng đợi lời bài hát.".into()))
        });
    }
    fn confirmation(&mut self, ctx: &egui::Context) {
        let Some(action) = self.confirm.clone() else {
            return;
        };
        egui::Window::new("Xác nhận")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(match action {
                    Confirmation::Track(_) => {
                        "Xóa bài khỏi thư viện? Tệp nhạc gốc được giữ nguyên."
                    }
                    Confirmation::Playlist(_) => {
                        "Xóa danh sách phát? Các bài trong thư viện được giữ nguyên."
                    }
                    Confirmation::Lyrics(_) => {
                        "Xóa lời được lưu trong ứng dụng? Tệp LRC gốc được giữ nguyên."
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Hủy").clicked() {
                        self.confirm = None;
                    }
                    if ui.button("Xác nhận xóa").clicked() {
                        match action.clone() {
                            Confirmation::Track(id) => self.db_action(|c| db::delete_track(c, id)),
                            Confirmation::Playlist(id) => {
                                self.db_action(|c| db::delete_playlist(c, id));
                                self.page = Page::Library;
                            }
                            Confirmation::Lyrics(track) => {
                                let app = self.services.clone();
                                self.task(async move {
                                    lyrics_jobs::lyrics_delete(app, track.into()).await?;
                                    Ok(Message::Done("Đã xóa lời trong ứng dụng.".into()))
                                });
                                self.editor.clear();
                                self.editor_dirty = false;
                                self.preview = None;
                                self.editor_revision += 1;
                            }
                        }
                        self.confirm = None;
                    }
                });
            });
    }
    fn island(&mut self, ctx: &egui::Context) {
        if !self.settings.island {
            return;
        }
        let id = egui::ViewportId::from_hash_of("dynamic-island");
        let size = if self.island_expanded {
            [390., 170.]
        } else {
            [250., 82.]
        };
        ctx.show_viewport_immediate(
            id,
            egui::ViewportBuilder::default()
                .with_title("Trantramy Dynamic Island")
                .with_inner_size(size)
                .with_decorations(false)
                .with_always_on_top()
                .with_resizable(false),
            |ctx, _| {
                ctx.send_viewport_cmd(egui::ViewportCommand::ContentProtected(
                    self.settings.stealth,
                ));
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.settings.island = false;
                    self.save_settings();
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    if ui
                        .add(
                            egui::Label::new(
                                self.player
                                    .current()
                                    .map_or("Trantramy", |t| t.title.as_str()),
                            )
                            .truncate()
                            .sense(Sense::click_and_drag()),
                        )
                        .drag_started()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .button(if self.player.phase == Phase::Playing {
                                "Dừng"
                            } else {
                                "Phát"
                            })
                            .clicked()
                        {
                            self.toggle();
                        }
                        if ui.button("Tiếp").clicked() {
                            self.next(false, false);
                        }
                        if ui
                            .button(if self.island_expanded {
                                "Gọn"
                            } else {
                                "Mở"
                            })
                            .clicked()
                        {
                            self.island_expanded = !self.island_expanded;
                        }
                    });
                    if self.island_expanded {
                        if let Some(t) = self.player.current()
                            && let Some(Ok(Some(r))) = self.lyric_cache.get(&t.path)
                        {
                            let parsed = lrc::parse(&r.content);
                            if !r.draft_only
                                && let Some(i) = lrc::active(&parsed.lines, self.audio.position())
                            {
                                ui.label(&parsed.lines[i].text);
                            }
                        }
                        if let Ok(wave) = self.audio.waveform.lock() {
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 35.),
                                Sense::hover(),
                            );
                            let painter = ui.painter_at(rect);
                            for (i, v) in wave.iter().enumerate() {
                                let x = rect.left() + i as f32 * rect.width() / 32.;
                                let h = v.min(1.) * rect.height();
                                painter.line_segment(
                                    [
                                        egui::pos2(x, rect.bottom()),
                                        egui::pos2(x, rect.bottom() - h),
                                    ],
                                    Stroke::new(4.0_f32, self.accent(ctx)),
                                );
                            }
                        }
                    }
                });
            },
        );
    }
}
impl eframe::App for MusicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(egui::ViewportCommand::ContentProtected(
            self.settings.stealth,
        ));
        if ctx.input(|i| i.viewport().close_requested()) && !self.persist_editor() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if !self.initialized {
            self.initialized = true;
            self.refresh_ai();
        }
        if let Some(palette) = self.palettes.iter().find(|p| p.id == self.settings.palette) {
            settings::apply(ctx, palette);
        }
        self.poll(ctx);
        if !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.toggle();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight))
                && let Err(e) = self
                    .audio
                    .seek((self.audio.position() + 5.).min(self.duration().max(0.)))
            {
                self.fail(e);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft))
                && let Err(e) = self.audio.seek((self.audio.position() - 5.).max(0.))
            {
                self.fail(e);
            }
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::K)) {
                self.open_karaoke();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
                self.karaoke_fullscreen = !self.karaoke_fullscreen;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.karaoke_fullscreen));
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.confirm = None;
                self.queue_open = false;
                self.desk_open = false;
                if self.karaoke_fullscreen {
                    self.karaoke_fullscreen = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                }
            }
        }
        let dropped = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect::<Vec<_>>()
        });
        if !dropped.is_empty() {
            self.scan(dropped);
        }
        egui::TopBottomPanel::bottom("player")
            .resizable(false)
            .show(ctx, |ui| {
                self.transport(ui);
                if !self.status.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        if self.status_error {
                            ui.colored_label(ui.visuals().error_fg_color, &self.status);
                        } else {
                            ui.label(&self.status);
                        }
                        if ui.small_button("Đóng").clicked() {
                            self.status.clear();
                        }
                    });
                }
            });
        if ctx.screen_rect().width() > 900. {
            egui::SidePanel::left("navigation")
                .exact_width(210.)
                .show(ctx, |ui| self.navigation(ui));
        }
        if self.queue_open {
            egui::SidePanel::right("queue-panel")
                .default_width(310.)
                .resizable(true)
                .show(ctx, |ui| self.queue_ui(ui));
        }
        if self.mini && self.page != Page::Karaoke {
            egui::SidePanel::right("mini-studio")
                .default_width(330.)
                .show(ctx, |ui| {
                    if ui.button("Mở rộng Studio").clicked() {
                        self.mini = false;
                        self.open_karaoke();
                    }
                    let r = self
                        .player
                        .current()
                        .and_then(|t| self.lyric_cache.get(&t.path))
                        .cloned();
                    self.singing_lyrics(ui, r, ui.available_height());
                });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(wallpaper) = &self.settings.wallpaper {
                let rect = ui.max_rect();
                egui::Image::new(uri(wallpaper))
                    .tint(Color32::from_white_alpha(
                        (self.settings.wallpaper_opacity * 255.) as u8,
                    ))
                    .paint_at(ui, rect);
            }
            self.header(ui);
            match self.page {
                Page::Home => self.home_ui(ui),
                Page::Library | Page::Liked | Page::Playlist(_) => self.library_ui(ui),
                Page::Online => self.online_ui(ui),
                Page::Detail => self.detail_ui(ui),
                Page::Lyrics => self.lyrics_ui(ui),
                Page::Karaoke => {
                    if !self.karaoke_window {
                        self.karaoke_ui(ui);
                    } else {
                        ui.heading("Studio đang mở trong cửa sổ riêng");
                        if ui.button("Đưa về cửa sổ chính").clicked() {
                            self.karaoke_window = false;
                        }
                    }
                }
                Page::Settings => self.settings_ui(ui),
            }
        });
        if self.karaoke_window {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("karaoke"),
                egui::ViewportBuilder::default()
                    .with_title("Trantramy · Studio Karaoke")
                    .with_inner_size([1100., 720.])
                    .with_min_inner_size([520., 400.]),
                |ctx, _| {
                    ctx.send_viewport_cmd(egui::ViewportCommand::ContentProtected(
                        self.settings.stealth,
                    ));
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.karaoke_window = false;
                    }
                    egui::CentralPanel::default().show(ctx, |ui| self.karaoke_ui(ui));
                },
            );
        }
        self.island(ctx);
        self.confirmation(ctx);
        ctx.request_repaint_after(Duration::from_millis(
            if self.player.phase == Phase::Playing {
                33
            } else if self.busy > 0 {
                100
            } else {
                500
            },
        ));
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.persist_editor();
        self.save_settings();
        let _ = lyrics_jobs::lyrics_pause(self.services.clone());
        self.audio.stop();
    }
}
fn page_name(page: Page) -> &'static str {
    match page {
        Page::Home => "Trang chủ",
        Page::Library => "Thư viện",
        Page::Liked => "Yêu thích",
        Page::Online => "Trực tuyến",
        Page::Playlist(_) => "Danh sách phát",
        Page::Detail => "Thông tin bài hát",
        Page::Lyrics => "Sửa lời bài hát",
        Page::Karaoke => "Studio Lời bài hát",
        Page::Settings => "Cài đặt",
    }
}
fn time(seconds: f64) -> String {
    let s = if seconds.is_finite() {
        seconds.max(0.) as u64
    } else {
        0
    };
    format!("{:02}:{:02}", s / 60, s % 60)
}
fn uri(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("file://") {
        path.into()
    } else {
        url::Url::from_file_path(path)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| format!("file://{path}"))
    }
}
fn online_track(t: &OnlineTrack) -> Track {
    Track {
        id: -1,
        path: t.url.clone(),
        title: t.title.clone(),
        artist: t.artist.clone(),
        album: t.album.clone(),
        duration: t.duration,
        cover_path: t.thumbnail.clone(),
        source: t.source.clone(),
        url: Some(t.url.clone()),
        liked: false,
    }
}
fn safe_name(s: &str) -> String {
    s.chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) { '_' } else { c })
        .collect()
}
fn copy_image(path: &std::path::Path, directory: &std::path::Path) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !["jpg", "jpeg", "png", "gif", "webp", "bmp"].contains(&extension.as_str()) {
        return Err("Định dạng ảnh không được hỗ trợ.".into());
    }
    let bytes = lyrics::read_limited(path, 20 * 1024 * 1024)?;
    image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    use sha2::{Digest, Sha256};
    let name = format!("{:x}.{extension}", Sha256::digest(&bytes));
    let dest = directory.join(name);
    std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().into_owned())
}
fn atomic_text(path: &std::path::Path, content: &str) -> Result<(), String> {
    use std::io::Write;
    let dir = path.parent().ok_or("Đường dẫn không hợp lệ.")?;
    let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.error.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drafts_survive_switches_and_only_current_success_clears_dirty_state() {
        let directory = tempfile::tempdir().unwrap();
        let services = AppHandle::open(crate::context::Paths {
            data: directory.path().join("data"),
            cache: directory.path().join("cache"),
        })
        .unwrap();
        db::upsert_track(
            &services.state::<db::Db>().0.lock().unwrap(),
            "a.wav",
            "A",
            "Artist",
            "Album",
            8.,
            None,
        )
        .unwrap();
        let mut app = MusicApp::headless(services);
        let a = app.tracks[0].clone();
        let mut b = a.clone();
        b.path = "b.wav".into();
        assert!(app.switch_editor(&a));
        app.editor = "[00:01.00]Giữ lời đang sửa".into();
        app.editor_dirty = true;
        assert!(app.switch_editor(&b));
        assert!(app.switch_editor(&a));
        assert!(app.editor_dirty);
        assert!(app.editor.contains("Giữ lời đang sửa"));
        let revision = app.editor_revision;
        app.tx
            .send(Message::Saved(a.path.clone(), revision - 1))
            .unwrap();
        app.poll(&egui::Context::default());
        assert!(app.editor_dirty);
        app.tx
            .send(Message::Failure("Simulated disk error".into()))
            .unwrap();
        app.poll(&egui::Context::default());
        assert!(app.editor_dirty);
        app.tx
            .send(Message::Saved(a.path.clone(), revision))
            .unwrap();
        app.poll(&egui::Context::default());
        assert!(!app.editor_dirty);
        assert!(!app.draft_path(&a.path).exists());
        app.lyric_cache.insert(
            a.path.clone(),
            Ok(Some(LyricsResult::new("old content".into(), "old", false))),
        );
        app.sync_editor();
        assert!(app.editor.contains("Giữ lời đang sửa"));
        app.editor_dirty = true;
        std::fs::remove_dir(app.services.path().data.join("native-drafts")).unwrap();
        std::fs::write(app.services.path().data.join("native-drafts"), b"blocked").unwrap();
        assert!(!app.switch_editor(&b));
        assert_eq!(app.editor_track, a.path);
    }
    #[test]
    fn renders_native_screens_without_network_or_audio_device() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _guard = runtime.enter();
        let directory = tempfile::tempdir().unwrap();
        let services = AppHandle::open(crate::context::Paths {
            data: directory.path().join("data"),
            cache: directory.path().join("cache"),
        })
        .unwrap();
        {
            let c = services.state::<db::Db>().inner().0.lock().unwrap();
            db::upsert_track(
                &c,
                "fixture.wav",
                "Bản nhạc tiếng Việt với tên dài",
                "Nghệ sĩ",
                "Album",
                60.,
                None,
            )
            .unwrap();
        }
        let mut app = MusicApp::headless(services);
        app.ai.automatic = false;
        app.services
            .state::<lyrics_jobs::LyricsService>()
            .config
            .lock()
            .unwrap()
            .automatic = false;
        app.services
            .state::<lyrics_jobs::LyricsService>()
            .config
            .lock()
            .unwrap()
            .network_lookup = false;
        let ctx = egui::Context::default();
        for width in [1440., 800., 520.] {
            for page in [
                Page::Home,
                Page::Library,
                Page::Liked,
                Page::Online,
                Page::Karaoke,
                Page::Settings,
            ] {
                app.page = page;
                let _output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(width, 400.),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            app.header(ui);
                            match page {
                                Page::Home => app.home_ui(ui),
                                Page::Library | Page::Liked => app.library_ui(ui),
                                Page::Online => app.online_ui(ui),
                                Page::Karaoke => app.karaoke_ui(ui),
                                Page::Settings => app.settings_ui(ui),
                                _ => {}
                            }
                        });
                    },
                );
            }
        }
        app.open_track(app.tracks[0].clone(), Page::Lyrics);
        app.lyric_cache.insert(
            "fixture.wav".into(),
            Ok(Some(LyricsResult::new(
                "[00:01.00]Xin chào".into(),
                "manual",
                false,
            ))),
        );
        app.sync_editor();
        assert!(app.editor.contains("Xin chào"));
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.lyrics_ui(ui));
        });
    }
}
