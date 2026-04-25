use globset::{Glob, GlobSet, GlobSetBuilder};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScopeError {
    #[error("invalid glob pattern '{pattern}': {source}")]
    InvalidGlob {
        pattern: String,
        source: globset::Error,
    },
}

/// URL scope checker with include/exclude glob patterns.
#[derive(Debug, Clone)]
pub struct ScopeChecker {
    include: GlobSet,
    exclude: GlobSet,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl ScopeChecker {
    /// Build a scope checker from include and exclude URL patterns.
    /// Patterns use glob syntax: `*` matches any chars, `**` matches path segments.
    pub fn new(
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<Self, ScopeError> {
        let include = build_globset(include_patterns)?;
        let exclude = build_globset(exclude_patterns)?;
        Ok(Self {
            include,
            exclude,
            include_patterns: include_patterns.to_vec(),
            exclude_patterns: exclude_patterns.to_vec(),
        })
    }

    /// Create a permissive scope that allows everything.
    pub fn allow_all() -> Self {
        Self {
            include: GlobSet::empty(),
            exclude: GlobSet::empty(),
            include_patterns: vec!["*".into()],
            exclude_patterns: vec![],
        }
    }

    /// Check if a URL is within scope.
    /// A URL is in-scope if it matches any include pattern (or includes is empty)
    /// AND does not match any exclude pattern.
    pub fn is_in_scope(&self, url: &str) -> bool {
        let included = self.include.is_empty() || self.include.is_match(url);
        let excluded = !self.exclude.is_empty() && self.exclude.is_match(url);
        included && !excluded
    }

    /// Filter a list of URLs to only those in scope.
    pub fn filter<'a, I>(&self, urls: I) -> Vec<&'a str>
    where
        I: IntoIterator<Item = &'a str>,
    {
        urls.into_iter().filter(|u| self.is_in_scope(u)).collect()
    }

    pub fn include_patterns(&self) -> &[String] {
        &self.include_patterns
    }

    pub fn exclude_patterns(&self) -> &[String] {
        &self.exclude_patterns
    }
}

fn build_globset(patterns: &[String]) -> Result<GlobSet, ScopeError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|e| ScopeError::InvalidGlob {
            pattern: pattern.clone(),
            source: e,
        })?;
        builder.add(glob);
    }
    builder.build().map_err(|e| ScopeError::InvalidGlob {
        pattern: "<builder>".into(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_all() {
        let scope = ScopeChecker::allow_all();
        assert!(scope.is_in_scope("https://example.com/api/users"));
    }

    #[test]
    fn test_include_only() {
        let scope = ScopeChecker::new(&["https://staging.example.com/*".into()], &[]).unwrap();
        assert!(scope.is_in_scope("https://staging.example.com/api"));
        assert!(!scope.is_in_scope("https://prod.example.com/api"));
    }

    #[test]
    fn test_include_and_exclude() {
        let scope = ScopeChecker::new(
            &["https://api.example.com/*".into()],
            &["https://api.example.com/admin/*".into()],
        )
        .unwrap();
        assert!(scope.is_in_scope("https://api.example.com/users"));
        assert!(!scope.is_in_scope("https://api.example.com/admin/settings"));
    }

    #[test]
    fn test_filter() {
        let scope = ScopeChecker::new(&["https://api.example.com/*".into()], &[]).unwrap();
        let urls = vec![
            "https://api.example.com/users",
            "https://other.com/api",
            "https://api.example.com/orders",
        ];
        let filtered = scope.filter(urls);
        assert_eq!(
            filtered,
            vec![
                "https://api.example.com/users",
                "https://api.example.com/orders"
            ]
        );
    }
}
