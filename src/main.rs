#![windows_subsystem = "windows"]

use calamine::{Data, Reader, open_workbook_auto};
use eframe::egui;
use pdfium_render::prelude::*;
use rfd::FileDialog;
use rusty_tesseract::{Args, Image};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{Receiver, channel};
use std::thread;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([750.0, 550.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MDWeave",
        options,
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.style()).clone();

            style.spacing.button_padding = egui::vec2(16.0, 8.0);
            style.spacing.item_spacing = egui::vec2(10.0, 10.0);

            style.visuals.window_rounding = egui::Rounding::same(12.0);
            style.visuals.menu_rounding = egui::Rounding::same(8.0);
            let widget_rounding = egui::Rounding::same(8.0);
            style.visuals.widgets.noninteractive.rounding = widget_rounding;
            style.visuals.widgets.inactive.rounding = widget_rounding;
            style.visuals.widgets.hovered.rounding = widget_rounding;
            style.visuals.widgets.active.rounding = widget_rounding;

            cc.egui_ctx.set_style(style);

            Box::new(MarkItDownApp::default())
        }),
    )
}

fn is_tesseract_installed() -> bool {
    Command::new("tesseract").arg("--version").output().is_ok()
}

struct MarkItDownApp {
    selected_file: Option<PathBuf>,
    markdown_output: String,

    is_processing: bool,
    receiver: Option<Receiver<String>>,

    show_tesseract_warning: bool,
}

impl Default for MarkItDownApp {
    fn default() -> Self {
        let tesseract_missing = !is_tesseract_installed();

        Self {
            selected_file: None,
            markdown_output: String::from("Hasil konversi markdown akan muncul di sini..."),
            is_processing: false,
            receiver: None,
            show_tesseract_warning: tesseract_missing,
        }
    }
}

impl eframe::App for MarkItDownApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if self.show_tesseract_warning {
            egui::Window::new("Peringatan OCR")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]) 
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Fitur OCR membutuhkan Tesseract. Silakan unduh dan instal dari:").size(14.0));
                    ui.add_space(5.0);
                    
                    ui.vertical_centered(|ui| {
                        ui.hyperlink_to(
                            "Tautan GitHub UB-Mannheim Tesseract Windows", 
                            "https://github.com/UB-Mannheim/tesseract/wiki"
                        );
                        ui.add_space(15.0);
                        if ui.button(egui::RichText::new("Tutup").size(14.0)).clicked() {
                            self.show_tesseract_warning = false;
                        }
                    });
                    ui.add_space(5.0);
                });
        }

        if let Some(rx) = &self.receiver {
            if let Ok(result) = rx.try_recv() {
                self.markdown_output = result;
                self.is_processing = false;
                self.receiver = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(15.0);
            
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("MDWeave").size(28.0).strong());
                ui.label(egui::RichText::new("Documents to Markdown Converter").size(16.0).italics());
            });
            
            ui.add_space(20.0);

            ui.add_enabled_ui(!self.is_processing, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.button(egui::RichText::new("📂 Pilih File Dokumen...").size(16.0)).clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter(
                                "Documents",
                                &["txt", "md", "csv", "xlsx", "xls", "pdf", "docx", "doc", "pptx", "ppt"],
                            )
                            .pick_file()
                        {
                            self.selected_file = Some(path.clone());
                            self.is_processing = true;
                            self.markdown_output = String::from(
                                "Memulai proses... (Jika ini adalah PDF tebal, OCR bisa memakan waktu)",
                            );

                            let (tx, rx) = channel();
                            self.receiver = Some(rx);

                            let ctx_clone = ctx.clone();
                            let path_clone = path.clone();

                            thread::spawn(move || {
                                let result = konversi_ke_markdown(&path_clone);
                                let _ = tx.send(result);
                                ctx_clone.request_repaint();
                            });
                        }
                    }
                });
            });

            ui.add_space(15.0);

            if self.is_processing {
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(egui::RichText::new("Menganalisis dokumen dan mengekstrak teks...").size(14.0));
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    if let Some(file) = &self.selected_file {
                        ui.label(egui::RichText::new(format!("File: {}", file.display())).strong());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("💾 Simpan .md").size(14.0)).clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("Markdown", &["md"])
                                .set_file_name("hasil_konversi.md")
                                .save_file()
                            {
                                if let Err(e) = std::fs::write(&path, &self.markdown_output) {
                                    self.markdown_output
                                        .push_str(&format!("\n\n[Gagal menyimpan file: {}]", e));
                                }
                            }
                        }

                        if ui.button(egui::RichText::new("📋 Salin").size(14.0)).clicked() {
                            ui.output_mut(|o| o.copied_text = self.markdown_output.clone());
                        }
                    });
                });
            }

            ui.separator();
            ui.add_space(5.0);

            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(10.0)) 
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.markdown_output)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(18) 
                                .frame(false), 
                        );
                    });
                });
        });
    }
}

fn konversi_ke_markdown(path: &PathBuf) -> String {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "txt" | "md" => {
            fs::read_to_string(path).unwrap_or_else(|err| format!("Gagal membaca teks: {}", err))
        }
        "csv" | "xlsx" | "xls" => parse_spreadsheet(path),
        "docx" => parse_docx(path),
        "doc" => "Format biner lawas (.doc) belum didukung secara native. Silakan buka file tersebut di Word dan lakukan 'Save As' ke format .docx terlebih dahulu.".to_string(),
        "pptx" => parse_pptx(path),
        "ppt" => "Format biner lawas (.ppt) belum didukung secara native. Silakan buka file tersebut di PowerPoint dan lakukan 'Save As' ke format .pptx terlebih dahulu.".to_string(),
        "pdf" => {
            let standard_text = parse_pdf(path);

            if standard_text.trim().len() < 50 {
                parse_pdf_ocr(path)
            } else {
                standard_text
            }
        }
        _ => format!("Format .{} belum didukung.", extension),
    }
}

fn parse_spreadsheet(path: &PathBuf) -> String {
    let mut workbook = match open_workbook_auto(path) {
        Ok(wb) => wb,
        Err(e) => return format!("Error membuka spreadsheet: {:?}", e),
    };

    let mut markdown = String::new();
    let sheet_names = workbook.sheet_names().to_owned();

    for sheet_name in sheet_names {
        markdown.push_str(&format!("## Sheet: {}\n\n", sheet_name));

        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut is_header = true;

            for row in range.rows() {
                markdown.push_str("|");
                for cell in row {
                    let cell_value = match cell {
                        Data::String(s) => s.replace("\n", " ").replace("|", "\\|"),
                        Data::Float(f) => f.to_string(),
                        Data::Int(i) => i.to_string(),
                        Data::Bool(b) => b.to_string(),
                        Data::Empty => String::new(),
                        _ => String::new(),
                    };
                    markdown.push_str(&format!(" {} |", cell_value));
                }
                markdown.push('\n');

                if is_header {
                    markdown.push_str("|");
                    for _ in row {
                        markdown.push_str("---|");
                    }
                    markdown.push('\n');
                    is_header = false;
                }
            }
        }
        markdown.push_str("\n\n");
    }

    markdown
}

fn parse_pdf(path: &PathBuf) -> String {
    match pdf_extract::extract_text(path) {
        Ok(text) => text,
        Err(_) => "Gagal mengekstrak teks dari PDF. Pastikan file PDF ini memiliki teks yang bisa diseleksi (bukan hasil scan gambar).".to_string(),
    }
}

fn parse_docx(path: &PathBuf) -> String {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return "Gagal membuka file DOCX.".to_string(),
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return "File bukan arsip DOCX yang valid.".to_string(),
    };

    let mut document_xml = match archive.by_name("word/document.xml") {
        Ok(f) => f,
        Err(_) => return "Struktur dokumen XML tidak ditemukan di dalam DOCX.".to_string(),
    };

    let mut xml_content = String::new();
    if document_xml.read_to_string(&mut xml_content).is_err() {
        return "Gagal membaca konten XML.".to_string();
    }

    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(&xml_content);
    let mut markdown = String::new();
    let mut in_text_node = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text_node = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:p" => {
                markdown.push_str("\n\n");
            }
            Ok(Event::Text(e)) if in_text_node => {
                if let Ok(text) = e.unescape() {
                    markdown.push_str(&text);
                }
                in_text_node = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => (),
        }
    }

    markdown
}

fn parse_pptx(path: &PathBuf) -> String {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return "Gagal membuka file PPTX.".to_string(),
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return "File bukan arsip PPTX yang valid.".to_string(),
    };

    let mut markdown = String::new();
    
    let mut slide_files: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_files.push(name);
            }
        }
    }

    slide_files.sort_by(|a, b| {
        let extract_num = |s: &str| -> u32 {
            s.replace("ppt/slides/slide", "")
             .replace(".xml", "")
             .parse()
             .unwrap_or(0)
        };
        extract_num(a).cmp(&extract_num(b))
    });

    for (index, slide_name) in slide_files.iter().enumerate() {
        markdown.push_str(&format!("## Slide {}\n\n", index + 1));

        let mut slide_xml = match archive.by_name(slide_name) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let mut xml_content = String::new();
        if slide_xml.read_to_string(&mut xml_content).is_ok() {
            use quick_xml::Reader;
            use quick_xml::events::Event;

            let mut reader = Reader::from_str(&xml_content);
            let mut in_text_node = false;
            let mut has_text = false;

            loop {
                match reader.read_event() {
                    Ok(Event::Start(ref e)) if e.name().as_ref() == b"a:t" => {
                        in_text_node = true;
                    }
                    Ok(Event::End(ref e)) if e.name().as_ref() == b"a:p" => {
                        markdown.push_str("\n\n");
                    }
                    Ok(Event::Text(e)) if in_text_node => {
                        if let Ok(text) = e.unescape() {
                            markdown.push_str(&text);
                            has_text = true;
                        }
                        in_text_node = false;
                    }
                    Ok(Event::Eof) => break,
                    Err(_) => break,
                    _ => (),
                }
            }

            if !has_text {
                markdown.push_str("*(Slide kosong atau hanya berisi gambar/grafik)*\n\n");
            }
        }
        
        markdown.push_str("---\n\n");
    }

    if markdown.is_empty() {
        return "Tidak ada teks yang dapat diekstrak dari file PPTX ini (mungkin hanya berisi gambar).".to_string();
    }

    markdown
}

fn parse_pdf_ocr(path: &PathBuf) -> String {
    let mut markdown = String::new();

    let bindings = match Pdfium::bind_to_system_library()
        .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))) 
    {
        Ok(b) => b,
        Err(_) => return "Gagal memuat PDFium. Pastikan file libpdfium.so atau pdfium.dll ada di direktori aplikasi.".to_string(),
    };

    let pdfium = Pdfium::new(bindings);

    let document = match pdfium.load_pdf_from_file(path, None) {
        Ok(doc) => doc,
        Err(_) => return "Gagal membuka file PDF untuk OCR.".to_string(),
    };

    let tesseract_args = Args {
        lang: "ind+eng".to_string(),
        ..Default::default()
    };

    for (index, page) in document.pages().iter().enumerate() {
        markdown.push_str(&format!("## Halaman {}\n\n", index + 1));

        let render_config = PdfRenderConfig::new().set_target_width(2000);

        if let Ok(bitmap) = page.render_with_config(&render_config) {
            let dynamic_image = bitmap.as_image();

            if let Ok(tesseract_image) = Image::from_dynamic_image(&dynamic_image) {
                match rusty_tesseract::image_to_string(&tesseract_image, &tesseract_args) {
                    Ok(text) => markdown.push_str(&text),
                    Err(e) => markdown.push_str(&format!("[Gagal OCR di halaman ini: {}]", e)),
                }
            }
        }
        markdown.push_str("\n\n---\n\n");
    }

    markdown
}
