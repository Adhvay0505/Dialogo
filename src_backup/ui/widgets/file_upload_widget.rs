use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Label, Button, ProgressBar, Image, 
    FileChooserNative, ResponseType, Frame, Overlay,
};
use libadwaita::prelude::*;
use std::path::PathBuf;

pub struct FileUploadWidget {
    widget: Frame,
    content_box: GtkBox,
    file_name_label: Label,
    file_size_label: Label,
    progress_bar: ProgressBar,
    status_label: Label,
    preview_image: Image,
    cancel_button: Button,
    retry_button: Button,
    file_path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Completed,
    Error(String),
    Cancelled,
}

impl FileUploadWidget {
    pub fn new() -> Self {
        let content_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        let file_info_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(12)
            .build();

        let file_icon = Image::builder()
            .icon_name("text-x-generic")
            .icon_size(gtk4::IconSize::Large)
            .build();

        let file_info_column = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let file_name_label = Label::builder()
            .label("No file selected")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["heading".to_string()])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();

        let file_size_label = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        file_info_column.append(&file_name_label);
        file_info_column.append(&file_size_label);

        let select_button = Button::builder()
            .label("Choose File")
            .css_classes(vec!["suggested-action".to_string()])
            .build();

        file_info_box.append(&file_icon);
        file_info_box.append(&file_info_column);
        file_info_box.append(&select_button);

        let progress_bar = ProgressBar::builder()
            .hexpand(true)
            .show_text(true)
            .text("")
            .visible(false)
            .build();

        let status_box = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .build();

        let status_label = Label::builder()
            .label("")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(vec!["caption".to_string()])
            .build();

        let cancel_button = Button::builder()
            .label("Cancel")
            .css_classes(vec!["destructive-action".to_string()])
            .visible(false)
            .build();

        let retry_button = Button::builder()
            .label("Retry")
            .css_classes(vec!["suggested-action".to_string()])
            .visible(false)
            .build();

        status_box.append(&status_label);
        status_box.append(&cancel_button);
        status_box.append(&retry_button);

        content_box.append(&file_info_box);
        content_box.append(&progress_bar);
        content_box.append(&status_box);

        let widget = Frame::builder()
            .css_classes(vec!["file-upload-widget".to_string()])
            .child(&content_box)
            .build();

        let mut upload_widget = Self {
            widget,
            content_box,
            file_name_label,
            file_size_label,
            progress_bar,
            status_label,
            preview_image: Image::new(),
            cancel_button: cancel_button.clone(),
            retry_button: retry_button.clone(),
            file_path: None,
        };

        // Connect button handlers
        select_button.connect_clicked(clone!(@strong upload_widget.widget as parent => move |_| {
            Self::show_file_chooser(&parent);
        }));

        cancel_button.connect_clicked(clone!(@strong mut upload_widget => move |_| {
            upload_widget.set_status(UploadStatus::Cancelled);
        }));

        retry_button.connect_clicked(clone!(@strong mut upload_widget => move |_| {
            if let Some(file_path) = &upload_widget.file_path {
                upload_widget.start_upload(file_path.clone());
            }
        }));

        upload_widget
    }

    pub fn set_file(&mut self, file_path: PathBuf) {
        self.file_path = Some(file_path.clone());
        
        // Update file info
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown file");
        
        let file_size = std::fs::metadata(&file_path)
            .map(|m| m.len())
            .map(|s| Self::format_size(s))
            .unwrap_or_default();

        self.file_name_label.set_label(file_name);
        self.file_size_label.set_label(&file_size);
        
        // Update icon based on file type
        let icon_name = Self::get_file_icon(file_path);
        // TODO: Update file icon

        self.set_status(UploadStatus::Pending);
    }

    pub fn start_upload(&mut self, file_path: PathBuf) {
        self.file_path = Some(file_path);
        self.set_status(UploadStatus::Uploading);
        
        // TODO: Start actual upload process
        self.start_fake_upload();
    }

    pub fn set_status(&mut self, status: UploadStatus) {
        match status {
            UploadStatus::Pending => {
                self.progress_bar.set_visible(false);
                self.cancel_button.set_visible(false);
                self.retry_button.set_visible(false);
                self.status_label.set_label("Ready to upload");
            }
            UploadStatus::Uploading => {
                self.progress_bar.set_visible(true);
                self.cancel_button.set_visible(true);
                self.retry_button.set_visible(false);
                self.status_label.set_label("Uploading...");
                self.progress_bar.set_fraction(0.0);
            }
            UploadStatus::Completed => {
                self.progress_bar.set_visible(true);
                self.cancel_button.set_visible(false);
                self.retry_button.set_visible(false);
                self.status_label.set_label("Upload completed");
                self.progress_bar.set_fraction(1.0);
                self.progress_bar.set_text("100%");
            }
            UploadStatus::Error(ref error) => {
                self.progress_bar.set_visible(false);
                self.cancel_button.set_visible(false);
                self.retry_button.set_visible(true);
                self.status_label.set_label(&format!("Error: {}", error));
            }
            UploadStatus::Cancelled => {
                self.progress_bar.set_visible(false);
                self.cancel_button.set_visible(false);
                self.retry_button.set_visible(true);
                self.status_label.set_label("Upload cancelled");
            }
        }
    }

    pub fn update_progress(&mut self, progress: f64, bytes_transferred: u64) {
        self.progress_bar.set_fraction(progress);
        self.progress_bar.set_text(&format!("{}% / {}", 
            (progress * 100.0) as i32, 
            Self::format_size(bytes_transferred)
        ));
        self.status_label.set_label(&format!("Uploading... {} / s", 
            Self::format_size(1024 * 1024) // Fake speed
        ));
    }

    fn start_fake_upload(&self) {
        let progress_bar = self.progress_bar.clone();
        let status_label = self.status_label.clone();
        let cancel_button = self.cancel_button.clone();
        
        glib::spawn_future_local(async move {
            let mut progress = 0.0;
            while progress < 1.0 {
                progress += 0.01;
                
                glib::timeout_future(std::time::Duration::from_millis(50)).await;
                
                progress_bar.set_fraction(progress);
                progress_bar.set_text(&format!("{}%", (progress * 100.0) as i32));
                status_label.set_label(&format!("Uploading... {}%", (progress * 100.0) as i32));
            }
            
            progress_bar.set_fraction(1.0);
            progress_bar.set_text("100%");
            status_label.set_label("Upload completed");
            cancel_button.set_visible(false);
        });
    }

    fn show_file_chooser(parent: &impl IsA<gtk4::Window>) {
        let dialog = gtk4::FileChooserNative::builder()
            .title("Choose File to Upload")
            .action(gtk4::FileChooserAction::Open)
            .transient_for(parent)
            .modal(true)
            .build();

        dialog.connect_response(None, |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        // TODO: Update the upload widget with the selected file
                    }
                }
            }
        });

        dialog.show();
    }

    fn get_file_icon(path: PathBuf) -> &'static str {
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => "image-x-generic",
                Some("pdf") => "application-pdf",
                Some("txt") | Some("md") => "text-x-generic",
                Some("zip") | Some("tar") | Some("gz") => "package-x-generic",
                Some("mp3") | Some("wav") | Some("flac") => "audio-x-generic",
                Some("mp4") | Some("avi") | Some("mkv") => "video-x-generic",
                _ => "text-x-generic",
            }
        } else {
            "text-x-generic"
        }
    }

    fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    pub fn get_widget(&self) -> &Frame {
        &self.widget
    }
}