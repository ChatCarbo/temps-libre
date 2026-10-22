use crate::Lang;
use regex::Regex;
use std::time::Duration;

pub struct Valuation {
    pub value: f64,
    pub note: String,
}

fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Estimation indicative : médiane des prix en euros trouvés dans les résultats
/// d'une recherche web « marque modèle montre occasion prix ».
pub fn estimate(brand: &str, model: &str, lang: Lang) -> Result<Valuation, String> {
    let query = match lang {
        Lang::Fr => format!("{brand} {model} montre occasion prix"),
        Lang::En => format!("{brand} {model} watch used price EUR"),
    };
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        url_encode(&query)
    );
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(12))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:128.0) Gecko/20100101 Firefox/128.0")
        .build();

    let html = agent
        .get(&url)
        .call()
        .map_err(|e| match lang {
            Lang::Fr => format!("Recherche impossible ({e})"),
            Lang::En => format!("Search failed ({e})"),
        })?
        .into_string()
        .map_err(|e| match lang {
            Lang::Fr => format!("Lecture impossible ({e})"),
            Lang::En => format!("Could not read the response ({e})"),
        })?;

    let tags = Regex::new(r"(?s)<[^>]+>").unwrap();
    let text = tags
        .replace_all(&html, " ")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&#x27;", "'");

    let re = Regex::new(
        r"(\d{1,3}(?:[ \u{a0}\u{202f}.,]\d{3})+|\d{2,6})(?:[,.](\d{1,2}))?\s?(?:€|EUR\b|euros?\b)",
    )
    .unwrap();

    let mut prices: Vec<f64> = re
        .captures_iter(&text)
        .filter_map(|c| {
            let int: String = c[1].chars().filter(|ch| ch.is_ascii_digit()).collect();
            let mut v: f64 = int.parse().ok()?;
            if let Some(d) = c.get(2) {
                v += d.as_str().parse::<f64>().ok()? / 10f64.powi(d.as_str().len() as i32);
            }
            Some(v)
        })
        .filter(|v| (30.0..=30_000.0).contains(v))
        .collect();

    if prices.len() < 3 {
        return Err(match lang {
            Lang::Fr => format!(
                "Trop peu de prix trouvés pour {brand} {model} ({})",
                prices.len()
            ),
            Lang::En => format!(
                "Too few prices found for {brand} {model} ({})",
                prices.len()
            ),
        });
    }

    prices.sort_by(|a, b| a.total_cmp(b));
    let n = prices.len();
    let cut = n / 5; // on écarte les 20 % extrêmes
    let core = &prices[cut..n - cut];
    let median = core[core.len() / 2];
    let value = (median / 5.0).round() * 5.0;

    Ok(Valuation {
        value,
        note: match lang {
            Lang::Fr => format!("médiane de {n} prix trouvés sur le web"),
            Lang::En => format!("median of {n} prices found on the web"),
        },
    })
}
