pub struct Cat {
    pub code: &'static str,
    pub desc: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/category_codes.rs"));

pub fn list() -> &'static [Cat] {
    CATS
}

pub fn resolve(pattern: &str) -> Result<Vec<&'static str>, String> {
    if let Some(prefix) = pattern.strip_suffix('*') {
        let matched: Vec<&str> = CATS
            .iter()
            .filter(|c| c.code.starts_with(prefix))
            .map(|c| c.code)
            .collect();
        if matched.is_empty() {
            return Err(format!("unknown category pattern '{}'", pattern));
        }
        Ok(matched)
    } else {
        CATS.iter()
            .find(|c| c.code == pattern)
            .map(|c| vec![c.code])
            .ok_or_else(|| {
                format!(
                    "unknown category '{}' (did you mean '{}*'?)",
                    pattern, pattern
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_exact() {
        let r = resolve("Lu").unwrap();
        assert_eq!(r, vec!["Lu"]);
    }

    #[test]
    fn resolve_wildcard_letters() {
        let r = resolve("L*").unwrap();
        for cat in &["Lu", "Ll", "Lt", "Lm", "Lo"] {
            assert!(r.contains(cat), "missing {cat}");
        }
    }

    #[test]
    fn resolve_wildcard_punctuation() {
        let r = resolve("P*").unwrap();
        for cat in &["Pc", "Pd", "Ps", "Pe", "Pi", "Pf", "Po"] {
            assert!(r.contains(cat), "missing {cat}");
        }
    }

    #[test]
    fn resolve_unknown_exact() {
        assert!(resolve("Xyz").is_err());
    }

    #[test]
    fn resolve_unknown_wildcard() {
        assert!(resolve("X*").is_err());
    }

    #[test]
    fn resolve_list_not_empty() {
        assert!(!list().is_empty());
    }
}
