use std::sync::Arc;
use std::{collections::HashMap, fs::read};

use egui::{FontData, FontDefinitions};
use font_kit::{family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource};

fn load_font_family(family_names: &[&str]) -> Option<Vec<u8>> {
    let system_source = SystemSource::new();

    for &name in family_names {
        match system_source
            .select_best_match(&[FamilyName::Title(name.to_string())], &Properties::new())
        {
            Ok(h) => match &h {
                Handle::Memory { bytes, .. } => {
                    return Some(bytes.to_vec());
                }
                Handle::Path { path, .. } => {
                    if let Ok(data) = read(path) {
                        return Some(data);
                    }
                }
            },
            Err(_) => (),
        }
    }

    None
}

pub fn load_system_fonts(mut fonts: FontDefinitions) -> FontDefinitions {
    let mut fontdb = HashMap::new();

    fontdb.insert(
        "simplified_chinese",
        vec![
            "Heiti SC",
            "Songti SC",
            "Noto Sans CJK SC",
            "Noto Sans SC",
            "WenQuanYi Zen Hei",
            "SimSun",
            "Noto Sans SC",
            "PingFang SC",
            "Source Han Sans CN",
        ],
    );

    fontdb.insert("korean", vec!["Source Han Sans KR"]);

    fontdb.insert(
        "arabic_fonts",
        vec![
            "Noto Sans Arabic",
            "Amiri",
            "Lateef",
            "Al Tarikh",
            "Segoe UI",
        ],
    );

    for (region, font_names) in fontdb {
        if let Some(font_data) = load_font_family(&font_names) {
            fonts
                .font_data
                .insert(region.to_owned(), Arc::new(FontData::from_owned(font_data)));

            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .push(region.to_owned());
        }
    }
    fonts
}
