#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use itertools::Itertools;
mod playfair;

const WINDOW_RECT: [f32; 2] = [1024.0, 600.0];

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_RECT)
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Playfair DNA",
        options,
        Box::new(|_cc| {
            // This gives us image support:

            Box::<App>::default()
        }),
    )
}

struct App {
    en_key: String,
    de_key: String,
    en_key_vec: String,
    de_key_vec: String,
    en_plain_text: String,
    de_plain_text: String,
    en_cipher: String,
    de_cipher: String,
    en_binary: String,
    de_binary: String,
    en_dna: String,
    de_dna: String,
    en_acids: String,
    de_acids: String,
    en_acids_after_playfair: String,
    de_acids_after_playfair: String,
    en_ambig_vec: Vec<u8>,
    de_ambig_vec: Vec<u8>,
    en_ambig: String,
    de_ambig: String,
    en_dna_after_playfair: String,
    de_dna_after_playfair: String,
    en_config: Config,
    de_config: Config,
}

struct Config {
    ambig_pos: bool,
    text_format: Encodings,
    // keep_whitespaces: bool,
}

#[derive(PartialEq)]
enum Encodings {
    UTF8,
    UTF16,
    //  UTF16le,
}

impl Encodings {
    fn string(&self) -> String {
        match self {
            Encodings::UTF8 => "UTF-8".to_string(),
            Encodings::UTF16 => "UTF-16 (big endian)".to_string(),
            // Encodings::UTF16le => "UTF-16 (little endian)".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ambig_pos: true,
            text_format: Encodings::UTF8,
            // keep_whitespaces: true,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            en_key: "".to_owned(),
            de_key: "".to_owned(),
            en_key_vec: "".to_owned(),
            de_key_vec: "".to_owned(),
            en_plain_text: "".to_owned(),
            en_cipher: "".to_owned(),
            de_plain_text: "".to_owned(),
            de_cipher: "".to_owned(),
            en_binary: "".to_owned(),
            de_binary: "".to_owned(),
            en_dna: "".to_owned(),
            de_dna: "".to_owned(),
            en_acids: "".to_owned(),
            de_acids: "".to_owned(),
            en_acids_after_playfair: "".to_owned(),
            de_acids_after_playfair: "".to_owned(),
            en_ambig_vec: Vec::new(),
            de_ambig_vec: Vec::new(),
            en_ambig: "".to_owned(),
            de_ambig: "".to_owned(),
            en_dna_after_playfair: "".to_owned(),
            de_dna_after_playfair: "".to_owned(),
            en_config: Config::default(),
            de_config: Config::default(),
        }
    }
}

impl App {
    fn encrypt(&mut self) {
        self.en_key_vec = playfair::generate_key_matrix(&self.en_key)
            .iter()
            .collect::<String>();
        self.en_binary = "".to_string();

        let bin: Vec<u8> = match self.en_config.text_format {
            Encodings::UTF8 => self.en_plain_text.as_bytes().to_vec(),
            Encodings::UTF16 => {
                playfair::utf16_to_binary(&self.en_plain_text.encode_utf16().collect::<Vec<u16>>())
            }
        };
        for character in bin.iter() {
            self.en_binary += &format!("{:b} ", character);
        }
        self.en_dna = playfair::binary_to_dna(&playfair::utf8_to_binary(&self.en_plain_text))
            .iter()
            .collect();

        let (acid, ambig) = playfair::dna_to_acids(&self.en_dna.chars().collect::<Vec<char>>());
        self.en_ambig_vec = ambig;
        self.en_acids = acid.iter().collect();
        self.en_ambig = "".to_string();

        if self.en_key_vec.len() == 25 {
            self.en_acids_after_playfair = playfair::encrypt(
                &self.en_key_vec.chars().collect::<Vec<char>>(),
                &self.en_acids,
                &mut self.en_ambig_vec,
            );
        } else {
            self.en_acids_after_playfair = "".to_string();
        }
        for byte in self.en_ambig_vec.iter() {
            self.en_ambig += &format!("{}", byte);
        }
        let acid_after_fair = self
            .en_acids_after_playfair
            .chars()
            .collect::<Vec<char>>()
            .clone();
        self.en_dna_after_playfair = playfair::acids_to_dna(
            &acid_after_fair,
            &vec![0u8; self.en_acids_after_playfair.len()],
        )
        .iter()
        .collect();

        self.en_cipher = playfair::dna_plus_ambig(
            &self.en_dna_after_playfair,
            &self.en_ambig_vec,
            self.en_config.ambig_pos,
        );
    }

    fn decrypt(&mut self) {
        self.de_key_vec = playfair::generate_key_matrix(&self.de_key)
            .iter()
            .collect::<String>();
        let (dna, ambig) = playfair::split_cipher(&self.de_cipher, self.de_config.ambig_pos);
        self.de_dna = dna.iter().collect();
        self.de_ambig_vec = ambig;
        self.de_ambig = "".to_string();
        if self.de_dna.len() != self.de_ambig_vec.len() * 3 {
            println!("something went wrong, dna/ambig length mismatch");
            return;
        }
        for byte in self.de_ambig_vec.iter() {
            self.de_ambig += &format!("{}", byte);
        }
        let (acids, _) = playfair::dna_to_acids(&self.de_dna.chars().collect_vec());
        if acids.len() != self.de_ambig_vec.len() {
            println!("something went wrong, acids/ambig length mismatch");
            return;
        }
        self.de_acids = acids.iter().collect();
        let unsanitized_acids = playfair::decrypt(&self.de_key_vec, &self.de_acids);
        self.de_acids_after_playfair =
            playfair::sanitize_acids(&unsanitized_acids, &self.de_ambig_vec);
        let mut sanitized_ambig_vector = self.de_ambig_vec.clone();
        playfair::sanitize_ambig(&mut sanitized_ambig_vector);
        self.de_dna_after_playfair = playfair::acids_to_dna(
            &self.de_acids_after_playfair.chars().collect::<Vec<char>>(),
            &sanitized_ambig_vector,
        )
        .iter()
        .collect();

        let bin = playfair::dna_to_binary(&self.de_dna_after_playfair);
        self.de_binary = "".to_string();
        for character in bin.iter() {
            self.de_binary += &format!("{:b} ", character);
        }
        self.de_plain_text = match String::from_utf8(bin) {
            Ok(v) => v,
            Err(e) => String::from(format!(
                "wrong binary format - check your key! \n err: {}",
                e
            )),
        };
    }
}

const TEXT_AREA_SIZE: egui::Vec2 = egui::vec2(WINDOW_RECT[0] / 2.5, 64.0);
const LAYOUT: egui::Layout = egui::Layout {
    main_dir: egui::Direction::TopDown,
    main_wrap: false,
    main_align: egui::Align::Center,
    main_justify: true,
    cross_align: egui::Align::Center,
    cross_justify: true,
};

const LEFT_UI_RECT: egui::Rect = egui::Rect {
    min: egui::Pos2 { x: 0.0, y: 0.0 },
    max: egui::Pos2 {
        x: WINDOW_RECT[0] / 2.0 - 5.0,
        y: WINDOW_RECT[1],
    },
};
const RIGHT_UI_RECT: egui::Rect = egui::Rect {
    min: egui::Pos2 {
        x: WINDOW_RECT[0] / 2.0 + 5.0,
        y: 0.0,
    },
    max: egui::Pos2 {
        x: WINDOW_RECT[0],
        y: WINDOW_RECT[1],
    },
};

const SEPARATOR_UI_RECT: egui::Rect = egui::Rect {
    min: egui::Pos2 {
        x: WINDOW_RECT[0] / 2.0 - 5.0,
        y: 0.0,
    },
    max: egui::Pos2 {
        x: WINDOW_RECT[0] / 2.0 + 5.0,
        y: WINDOW_RECT[1],
    },
};

fn create_input_box(ui: &mut egui::Ui, value: &mut String, name: &str, id: &str, offset: f32) {
    ui.vertical_centered(|ui| {
        let link = ui.link(format!("{}:", name));
        if link.clicked() {
            // Copy to clipboard
            let mut clipboard_ctx = ClipboardContext::new().unwrap();
            clipboard_ctx.set_contents(value.to_owned()).unwrap();
        }
        if link.hovered() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("copy_tooltip"), |ui| {
                ui.label("copy");
            });
        }
        ui.horizontal(|ui| {
            ui.add_space(offset);
            egui::ScrollArea::vertical()
                .id_source(id)
                .max_width(TEXT_AREA_SIZE.x)
                .max_height(TEXT_AREA_SIZE.y)
                .show(ui, |ui| {
                    ui.add_enabled(
                        false,
                        egui::TextEdit::multiline(value).min_size(TEXT_AREA_SIZE),
                    );
                });
        });

        ui.add_space(10.0);
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
            ui.style_mut().spacing.interact_size.y = 1.0;

            // encrypt
            let mut _ui_l = ui.child_ui(LEFT_UI_RECT, LAYOUT);
            let l_ui_offset = (_ui_l.max_rect().max.x - TEXT_AREA_SIZE.x) / 2.0;
            egui::ScrollArea::vertical()
                .id_source("cipher_scroll_area")
                .max_height(WINDOW_RECT[0])
                .show(&mut _ui_l, |_ui_l| {
                    _ui_l.vertical_centered(|_ui_l| {
                        _ui_l.add_space(10.0);
                        _ui_l.heading("Encrypt");
                        egui::CollapsingHeader::new("Controls")
                            .id_source("cipher_controls")
                            .show(_ui_l, |_ui_l| {
                                _ui_l.label("Ambiguity: ");
                                _ui_l.horizontal(|_ui_l| {
                                    _ui_l.add_space(60.0);
                                    if _ui_l
                                        .add(egui::RadioButton::new(
                                            self.en_config.ambig_pos,
                                            "Before",
                                        ))
                                        .clicked()
                                    {
                                        self.en_config.ambig_pos = true;
                                        self.en_cipher = playfair::dna_plus_ambig(
                                            &self.en_dna_after_playfair,
                                            &self.en_ambig_vec,
                                            self.en_config.ambig_pos,
                                        );
                                    }
                                    _ui_l.add_space(10.0);
                                    if _ui_l
                                        .add(egui::RadioButton::new(
                                            !self.en_config.ambig_pos,
                                            "After",
                                        ))
                                        .clicked()
                                    {
                                        self.en_config.ambig_pos = false;
                                        self.en_cipher = playfair::dna_plus_ambig(
                                            &self.en_dna_after_playfair,
                                            &self.en_ambig_vec,
                                            self.en_config.ambig_pos,
                                        );
                                    }
                                });
                                _ui_l.label("Encoding: ");
                                _ui_l.horizontal(|_ui_l| {
                                    _ui_l.add_space(60.0);
                                    egui::ComboBox::from_id_source("encodings")
                                        .width(150.0)
                                        .selected_text(self.en_config.text_format.string())
                                        .show_ui(_ui_l, |_ui_l| {
                                            _ui_l.selectable_value(
                                                &mut self.en_config.text_format,
                                                Encodings::UTF8,
                                                "UTF-8",
                                            );
                                            _ui_l.selectable_value(
                                                &mut self.en_config.text_format,
                                                Encodings::UTF16,
                                                "UTF-16 (big endian)",
                                            );
                                        });
                                })
                            });
                        _ui_l.add_space(10.0);
                        let key = _ui_l.add(
                            egui::TextEdit::singleline(&mut self.en_key)
                                .min_size(egui::vec2(TEXT_AREA_SIZE.x / 2.0, 20.0))
                                .hint_text("Key"),
                        );
                        if key.changed() && !self.en_plain_text.is_empty() {
                            self.encrypt();
                        }
                        if key.hovered() && !self.en_key_vec.is_empty() {
                            //TODO show key matrix
                            egui::show_tooltip_at_pointer(
                                _ui_l.ctx(),
                                egui::Id::new("copy_tooltip"),
                                |_ui_l| {
                                    ui.set_min_width(50.0);
                                    for i in (0..self.en_key_vec.len()).step_by(5) {
                                        _ui_l.add(egui::Label::new(
                                            egui::RichText::new(format!(
                                                " {} {} {} {} {} ",
                                                self.en_key_vec.chars().collect::<Vec<char>>()[i],
                                                self.en_key_vec.chars().collect::<Vec<char>>()
                                                    [i + 1],
                                                self.en_key_vec.chars().collect::<Vec<char>>()
                                                    [i + 2],
                                                self.en_key_vec.chars().collect::<Vec<char>>()
                                                    [i + 3],
                                                self.en_key_vec.chars().collect::<Vec<char>>()
                                                    [i + 4],
                                            ))
                                            .text_style(egui::TextStyle::Monospace),
                                        ));
                                    }
                                },
                            );
                        }
                        _ui_l.add_space(10.0);
                        _ui_l.horizontal(|_ui_l| {
                            _ui_l.add_space(l_ui_offset);

                            egui::ScrollArea::vertical()
                                .id_source("cipher_scroll")
                                .max_width(TEXT_AREA_SIZE.x)
                                .max_height(TEXT_AREA_SIZE.y)
                                .show(_ui_l, |_ui_l| {
                                    let text_area = _ui_l.add(
                                        egui::TextEdit::multiline(&mut self.en_plain_text)
                                            .min_size(TEXT_AREA_SIZE)
                                            .hint_text("Plain Text"),
                                    );
                                    if text_area.changed() {
                                        self.encrypt();
                                    }
                                });
                        });
                        _ui_l.add_space(10.0);
                        create_input_box(
                            _ui_l,
                            &mut self.en_cipher,
                            "Cipher (encrypted DNA + Ambig)",
                            "cipher_en",
                            l_ui_offset,
                        );
                        _ui_l.add_space(10.0);
                        egui::CollapsingHeader::new("Extra").show_unindented(_ui_l, |_ui_l| {
                            _ui_l.vertical_centered(|_ui_l| {
                                create_input_box(
                                    _ui_l,
                                    &mut self.en_binary,
                                    "Binary",
                                    "bin_en",
                                    l_ui_offset,
                                );
                                create_input_box(
                                    _ui_l,
                                    &mut self.en_dna,
                                    "DNA",
                                    "dna_en",
                                    l_ui_offset,
                                );
                                create_input_box(
                                    _ui_l,
                                    &mut self.en_acids,
                                    "Acids",
                                    "acid_en",
                                    l_ui_offset,
                                );
                                create_input_box(
                                    _ui_l,
                                    &mut self.en_ambig,
                                    "Ambig",
                                    "ambig_en",
                                    l_ui_offset,
                                );
                                create_input_box(
                                    _ui_l,
                                    &mut self.en_acids_after_playfair,
                                    "encypted Acids",
                                    "en_acid_en",
                                    l_ui_offset,
                                );
                                create_input_box(
                                    _ui_l,
                                    &mut self.en_dna_after_playfair,
                                    "encrypted DNA",
                                    "en_dna_en",
                                    l_ui_offset,
                                );
                            });
                        });

                        _ui_l.add_space(10.0);
                    });
                });

            // separator
            let mut _ui_separator = ui.child_ui(SEPARATOR_UI_RECT, LAYOUT);
            _ui_separator.vertical_centered(|_ui_separator| {
                _ui_separator.add(egui::Separator::default().vertical());
            });

            // decrypt
            let mut _ui_r = ui.child_ui(RIGHT_UI_RECT, LAYOUT);
            let r_ui_offset =
                (_ui_r.max_rect().max.x - _ui_r.max_rect().min.x - TEXT_AREA_SIZE.x) / 2.0;
            egui::ScrollArea::vertical()
                .id_source("decipher_scroll_area")
                .max_height(WINDOW_RECT[0])
                .show(&mut _ui_r, |_ui_r| {
                    _ui_r.vertical_centered(|_ui_r| {
                        _ui_r.add_space(10.0);
                        _ui_r.heading("Decrypt");
                        egui::CollapsingHeader::new("Controls")
                            .id_source("decipher_controls")
                            .show(_ui_r, |_ui_r| {
                                _ui_r.label("Ambiguity: ");
                                _ui_r.horizontal(|_ui_r| {
                                    _ui_r.add_space(60.0);
                                    if _ui_r
                                        .add(egui::RadioButton::new(
                                            self.de_config.ambig_pos,
                                            "Before",
                                        ))
                                        .clicked()
                                    {
                                        self.de_config.ambig_pos = true;
                                    }
                                    _ui_r.add_space(10.0);
                                    if _ui_r
                                        .add(egui::RadioButton::new(
                                            !self.de_config.ambig_pos,
                                            "After",
                                        ))
                                        .clicked()
                                    {
                                        self.de_config.ambig_pos = false;
                                    }
                                });
                                _ui_r.label("Encoding: ");
                                _ui_r.horizontal(|_ui_r| {
                                    _ui_r.add_space(60.0);
                                    egui::ComboBox::from_id_source("de_encodings")
                                        .width(150.0)
                                        .selected_text(self.de_config.text_format.string())
                                        .show_ui(_ui_r, |_ui_r| {
                                            _ui_r.selectable_value(
                                                &mut self.de_config.text_format,
                                                Encodings::UTF8,
                                                "UTF-8",
                                            );
                                            _ui_r.selectable_value(
                                                &mut self.de_config.text_format,
                                                Encodings::UTF16,
                                                "UTF-16 (big endian)",
                                            );
                                        });
                                })
                            });
                        _ui_r.add_space(10.0);
                        let key = _ui_r.add(
                            egui::TextEdit::singleline(&mut self.de_key)
                                .min_size(egui::vec2(TEXT_AREA_SIZE.x / 2.0, 20.0))
                                .hint_text("Key"),
                        );
                        if key.changed() && !self.de_cipher.is_empty() {
                            self.decrypt();
                        }
                        if key.hovered() && !self.de_key_vec.is_empty() {
                            //TODO show key matrix
                            egui::show_tooltip_at_pointer(
                                _ui_r.ctx(),
                                egui::Id::new("copy_tooltip"),
                                |_ui_r| {
                                    ui.set_min_width(50.0);
                                    for i in (0..self.de_key_vec.len()).step_by(5) {
                                        _ui_r.add(egui::Label::new(
                                            egui::RichText::new(format!(
                                                " {} {} {} {} {} ",
                                                self.de_key_vec.chars().collect::<Vec<char>>()[i],
                                                self.de_key_vec.chars().collect::<Vec<char>>()
                                                    [i + 1],
                                                self.de_key_vec.chars().collect::<Vec<char>>()
                                                    [i + 2],
                                                self.de_key_vec.chars().collect::<Vec<char>>()
                                                    [i + 3],
                                                self.de_key_vec.chars().collect::<Vec<char>>()
                                                    [i + 4],
                                            ))
                                            .text_style(egui::TextStyle::Monospace),
                                        ));
                                    }
                                },
                            );
                        }
                        _ui_r.add_space(10.0);
                        _ui_r.horizontal(|_ui_r| {
                            _ui_r.add_space(r_ui_offset);

                            egui::ScrollArea::vertical()
                                .id_source("scroll_decipher_input")
                                .max_width(TEXT_AREA_SIZE.x)
                                .max_height(TEXT_AREA_SIZE.y)
                                .show(_ui_r, |_ui_r| {
                                    let text_area = _ui_r.add(
                                        egui::TextEdit::multiline(&mut self.de_cipher)
                                            .min_size(TEXT_AREA_SIZE)
                                            .hint_text("Cipher (encrypted DNA + Ambig)"),
                                    );
                                    if text_area.changed() {
                                        self.decrypt();
                                    }
                                });
                        });
                        _ui_r.add_space(10.0);
                        create_input_box(
                            _ui_r,
                            &mut self.de_plain_text,
                            "Plain Text",
                            "text_de",
                            r_ui_offset,
                        );
                        egui::CollapsingHeader::new("Extra")
                            .id_source("decipher_extra")
                            .show_unindented(_ui_r, |_ui_r| {
                                _ui_r.vertical_centered(|_ui_r| {
                                    create_input_box(
                                        _ui_r,
                                        &mut self.de_dna,
                                        "DNA",
                                        "dna_de",
                                        r_ui_offset,
                                    );
                                    create_input_box(
                                        _ui_r,
                                        &mut self.de_ambig,
                                        "Ambig",
                                        "ambig_de",
                                        r_ui_offset,
                                    );
                                    create_input_box(
                                        _ui_r,
                                        &mut self.de_acids,
                                        "Acids",
                                        "acid_de",
                                        r_ui_offset,
                                    );
                                    create_input_box(
                                        _ui_r,
                                        &mut self.de_acids_after_playfair,
                                        "decrypted Acids",
                                        "de_acid_de",
                                        r_ui_offset,
                                    );
                                    create_input_box(
                                        _ui_r,
                                        &mut self.de_dna_after_playfair,
                                        "decrypted DNA",
                                        "de_dna_de",
                                        r_ui_offset,
                                    );
                                    create_input_box(
                                        _ui_r,
                                        &mut self.de_binary,
                                        "Binary",
                                        "bin_de",
                                        r_ui_offset,
                                    );
                                });
                            });

                        _ui_r.add_space(10.0);
                    });
                });
        });
    }
}
