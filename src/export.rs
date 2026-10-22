use crate::{drive_label, t, Lang, Watch};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

const CSS: &str = r##"
:root{--bg:#f8f9fa;--card:#ffffff;--line:#e2e8f0;--gold:#b8860b;--txt:#1a202c;--mut:#718096}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--txt);font-family:system-ui,Segoe UI,sans-serif}
header{padding:32px 24px 8px;text-align:center}
h1{margin:0;font-size:2rem;color:var(--gold);letter-spacing:.08em}
.sub{color:var(--mut);margin-top:6px}
.bar{display:flex;gap:10px;justify-content:center;padding:18px 24px;flex-wrap:wrap}
input,select{background:var(--card);color:var(--txt);border:1px solid var(--line);border-radius:8px;padding:9px 12px;font-size:1rem}
#grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(270px,1fr));gap:18px;padding:8px 24px 40px;max-width:1300px;margin:auto}
.card{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:14px;box-shadow:0 1px 3px rgba(0,0,0,0.05)}
.card:hover{border-color:var(--gold)}
.photo{width:100%;aspect-ratio:1;object-fit:contain;border-radius:8px;background:#f1f5f9}
.noimg{display:flex;align-items:center;justify-content:center;font-size:4rem;color:var(--mut)}
.brand{display:flex;align-items:center;justify-content:center;gap:8px;margin-top:10px;font-weight:700}
.brand img{width:22px;height:22px;object-fit:contain}
.model{color:var(--mut);margin:2px 0 10px;text-align:center}
.specs{display:grid;grid-template-columns:auto 1fr;gap:4px 12px;font-size:.85rem;margin:0;padding-top:10px;border-top:1px solid var(--line)}
.specs dt{color:var(--mut)}
.specs dd{margin:0}
footer{text-align:center;color:var(--mut);font-size:.8rem;padding:0 0 30px}
"##;

const JS: &str = r##"
const q=document.getElementById('q'),s=document.getElementById('sort'),g=document.getElementById('grid');
function upd(){
  const t=q.value.toLowerCase(), k=s.value, cs=[...g.children];
  cs.sort((a,b)=>(a.dataset[k]||'').localeCompare(b.dataset[k]||''))
    .forEach(c=>g.appendChild(c));
  cs.forEach(c=>c.style.display=c.textContent.toLowerCase().includes(t)?'':'none');
}
q.oninput=s.onchange=upd; upd();
"##;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn data_uri(path: &str, max: u32, as_png: bool) -> Option<String> {
    if !Path::new(path).exists() {
        return None;
    }
    let img = image::open(path).ok()?.thumbnail(max, max);
    let mut buf = Cursor::new(Vec::new());
    let mime = if as_png {
        img.write_to(&mut buf, image::ImageFormat::Png).ok()?;
        "image/png"
    } else {
        image::DynamicImage::ImageRgb8(img.to_rgb8())
            .write_to(&mut buf, image::ImageFormat::Jpeg)
            .ok()?;
        "image/jpeg"
    };
    Some(format!(
        "data:{mime};base64,{}",
        STANDARD.encode(buf.into_inner())
    ))
}

pub fn export_html(
    watches: &[Watch],
    lang: Lang,
) -> Result<Option<PathBuf>, String> {
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(t(
            lang,
            "ma_collection.html",
            "my_collection.html",
            "mi_coleccion.html",
        ))
        .add_filter("HTML", &["html"])
        .save_file()
    else {
        return Ok(None);
    };

    let mut cards = String::new();
    for w in watches {
        cards.push_str(&format!(
            "<article class=\"card\" data-brand=\"{}\" data-model=\"{}\">",
            esc(&w.brand),
            esc(&w.model)
        ));

        match w.image_path.as_deref().and_then(|p| data_uri(p, 640, false)) {
            Some(uri) => cards.push_str(&format!(
                "<img class=\"photo\" alt=\"{}\" src=\"{}\">",
                esc(&w.model),
                uri
            )),
            None => cards.push_str("<div class=\"photo noimg\">⌚</div>"),
        }

        cards.push_str("<div class=\"brand\">");
        if let Some(uri) = w.brand_image_path.as_deref().and_then(|p| data_uri(p, 96, true)) {
            cards.push_str(&format!("<img alt=\"\" src=\"{uri}\">"));
        }
        cards.push_str(&format!("{}</div>", esc(&w.brand)));
        cards.push_str(&format!("<div class=\"model\">{}</div>", esc(&w.model)));

        let specs: [(&str, String); 5] = [
            (t(lang, "Cadran", "Dial", "Esfera"), w.dial_color.clone()),
            (t(lang, "Boîtier", "Case", "Caja"), w.case_material.clone()),
            (t(lang, "Mouvement", "Movement", "Movimiento"), w.movement.clone()),
            (t(lang, "Entraînement", "Drive", "Mecanismo"), drive_label(lang, &w.drive_type)),
            (t(lang, "Diamètre", "Diameter", "Diámetro"), w.case_diameter.clone()),
        ];
        cards.push_str("<dl class=\"specs\">");
        for (label, value) in specs {
            if !value.trim().is_empty() {
                cards.push_str(&format!("<dt>{}</dt><dd>{}</dd>", label, esc(&value)));
            }
        }
        cards.push_str("</dl>");
        cards.push_str("</article>\n");
    }

    let html = format!(
        "<!doctype html>\n<html lang=\"{code}\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>{title}</title><style>{CSS}</style></head><body>\
<header><h1>⏳ {h1}</h1><div class=\"sub\">{count} {unit}</div></header>\
<div class=\"bar\"><input id=\"q\" placeholder=\"🔍 {search}\">\
<select id=\"sort\"><option value=\"brand\">{by_brand}</option>\
<option value=\"model\">{by_model}</option></select></div>\
<main id=\"grid\">\n{cards}</main>\
<footer>{footer}</footer><script>{JS}</script></body></html>",
        code = lang.code(),
        title = t(
            lang,
            "Ma collection de montres",
            "My watch collection",
            "Mi colección de relojes"
        ),
        h1 = "TEMPS LIBRE",
        count = watches.len(),
        unit = t(lang, "montres", "watches", "relojes"),
        search = t(lang, "Rechercher…", "Search…", "Buscar…"),
        by_brand = t(lang, "Marque A→Z", "Brand A→Z", "Marca A→Z"),
        by_model = t(lang, "Modèle A→Z", "Model A→Z", "Modelo A→Z"),
        footer = t(
            lang,
            "Généré avec Temps Libre",
            "Generated with Temps Libre",
            "Generado con Temps Libre"
        ),
    );

    fs::write(&path, html).map_err(|e| match lang {
        Lang::Fr => format!("Écriture impossible : {e}"),
        Lang::En => format!("Could not write the file: {e}"),
        Lang::Es => format!("No se pudo escribir el archivo: {e}"),
    })?;
    Ok(Some(path))
}