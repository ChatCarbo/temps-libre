use std::{fs, io::Read, path::PathBuf, time::Duration};

/// Cherche le logo d'une marque (favicon de son site) et le met en cache dans ./logos/
pub fn fetch_logo(brand: &str) -> Option<PathBuf> {
    let raw_brand = brand.trim();
    if raw_brand.is_empty() {
        return None;
    }

    let slug: String = raw_brand
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    if slug.is_empty() {
        return None;
    }

    let dest = PathBuf::from("logos").join(format!("{slug}.png"));
    if dest.exists() {
        return Some(dest);
    }
    let _ = fs::create_dir_all("logos");

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build();

    // 1. Essai de correspondances directes pour les micro-marques connues (souvent difficiles à deviner automatiquement)
    let explicit_domains = match slug.as_str() {
        "addiesdive" => vec!["addiesdive.com"],
        "steeldive" | "steeldives" => vec!["steeldive.com", "steeldivewatch.net"],
        "sanmartin" => vec!["sanmartinwatches.com"],
        "seestern" => vec!["seesternwatches.com"],
        "heimdallr" => vec!["heimdallrwatch.com"],
        "cronos" => vec!["cronoswatches.com"],
        "watchdives" => vec!["watchdives.com"],
        "Sugess" | "sugess" => vec!["sugesswatch.com"],
        "baltany" => vec!["baltany.com"],
        "phiiswatch" | "phoebis" => vec!["phiiswatch.com"],
        _ => vec![],
    };

    // 2. Génération dynamique de domaines potentiels (élargie aux TLDs internationaux et asiatiques)
    let brand_no_space = raw_brand.to_lowercase().replace(' ', "");
    let brand_dash = raw_brand.to_lowercase().replace(' ', "-");

    let mut domains = Vec::new();
    for d in explicit_domains {
        domains.push(d.to_string());
    }

    for base in &[slug.as_str(), brand_no_space.as_str(), brand_dash.as_str()] {
        if !base.is_empty() {
            for tld in &["com", "watch", "net", "cn", "com.cn", "co", "io", "jp", "de"] {
                domains.push(format!("{base}.{tld}"));
                domains.push(format!("{base}watch.{tld}"));
                domains.push(format!("{base}watches.{tld}"));
            }
        }
    }

    // Teste chaque domaine pour trouver un favicon valide
    for domain in domains {
        // Vérifie rapidement si le domaine répond
        match agent.head(&format!("https://{domain}")).call() {
            Ok(_) | Err(ureq::Error::Status(_, _)) => {}
            Err(_) => continue,
        }

        let url = format!("https://www.google.com/s2/favicons?domain={domain}&sz=128");
        if let Ok(resp) = agent.get(&url).call() {
            let mut bytes = Vec::new();
            if resp
                .into_reader()
                .take(2_000_000)
                .read_to_end(&mut bytes)
                .is_ok()
                && bytes.len() > 200 // Ignore les favicons par défaut vides (souvent de très petite taille)
                && fs::write(&dest, &bytes).is_ok()
            {
                return Some(dest);
            }
        }
    }

    None
}