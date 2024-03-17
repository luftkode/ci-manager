use time::{format_description::well_known, OffsetDateTime};

use crate::*;

/// Type representing a date in the format `YYYY-MM-DD`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Date { year, month, day } = self;
        write!(f, "{year}-{month:02}-{day:02}")
    }
}

/// Filter an element by its creation or update date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateFilter {
    Created(Date),
    Updated(Date),
    None,
}

impl fmt::Display for DateFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateFilter::Created(date) => write!(f, "created:{date}"),
            DateFilter::Updated(date) => write!(f, "updated:{date}"),
            DateFilter::None => f.write_str(""), // No date filter
        }
    }
}

/// Filter an element by its labels. This is a type-safe way to create a filter string for the GitHub API.
///
/// # Example
///
/// ```
/// # use ci_manager::ci_provider::util::LabelFilter;
///
/// // Get elements with the label "bug"
/// let label_filter = LabelFilter::Any(["bug"]);
/// assert_eq!(label_filter.to_string(), r#"label:"bug""#);
/// ```
/// ```
/// # use ci_manager::ci_provider::util::LabelFilter;
/// // Only get elements with the labels "bug" and "help wanted"
/// let label_filter = LabelFilter::All(["bug", "help wanted"]);
/// assert_eq!(label_filter.to_string(), r#"label:"bug" label:"help wanted""#);
/// ```
/// ```
/// # use ci_manager::ci_provider::util::LabelFilter;
/// // Do not filter by labels
/// let label_filter = LabelFilter::none();
/// assert_eq!(label_filter.to_string(), "");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelFilter<I, S>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<str> + fmt::Display + fmt::Debug,
{
    /// Any of the labels must be present.
    Any(I),
    /// All labels must be present.
    All(I),
    /// No label filter.
    ///
    /// # Note: Use the `none()` method to create this variant.
    /// Or deal with the type complexity manually :).
    None(PhantomData<I>),
}

impl LabelFilter<Vec<String>, String> {
    /// Default label filter, does not filter by labels
    pub fn none() -> Self {
        LabelFilter::None(PhantomData)
    }
}

impl<I, S> fmt::Display for LabelFilter<I, S>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<str> + fmt::Display + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabelFilter::Any(labels) => write!(
                f,
                "label:{}",
                labels
                    .clone()
                    .into_iter()
                    .map(|l| format!("\"{l}\""))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            LabelFilter::All(labels) => write!(
                f,
                "{}",
                labels
                    .clone()
                    .into_iter()
                    .map(|l| format!("label:\"{l}\""))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            LabelFilter::None(_) => f.write_str(""), // No label filter
        }
    }
}

/// Extract the timestamp from a log string.
///
/// # Example
///
/// ```
/// # use ci_manager::ci_provider::util::timestamp_from_log;
/// # use pretty_assertions::assert_eq;
///
/// let log = "2024-01-17T11:23:18.0396058Z This is a log message";
/// let timestamp = timestamp_from_log(log).unwrap();
/// assert_eq!(timestamp.to_string(), "2024-01-17 11:23:18.0396058 +00:00:00");
/// ```
///
/// # Errors
/// - If the timestamp could not be extracted from the log.
/// - If the timestamp could not be parsed.
pub fn timestamp_from_log(log: &str) -> Result<OffsetDateTime> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z").unwrap());
    let captures = RE.captures(log);
    if let Some(captures) = captures {
        let timestamp = captures
            .get(0)
            .context("Could not extract timestamp")?
            .as_str();
        OffsetDateTime::parse(timestamp, &well_known::Iso8601::DEFAULT)
            .with_context(|| format!("Could not parse timestamp: {timestamp}"))
    } else {
        bail!("Could not extract timestamp from log: {log}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobLog {
    pub name: String,
    pub content: String,
}

impl JobLog {
    pub fn new(name: String, content: String) -> Self {
        Self { name, content }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_date_display() {
        let date = Date {
            year: 2021,
            month: 6,
            day: 2,
        };
        assert_eq!(date.to_string(), "2021-06-02");
    }

    #[test]
    fn test_date_filter_display() {
        let date = Date {
            year: 2021,
            month: 6,
            day: 2,
        };
        let date_filter = DateFilter::Created(date);
        assert_eq!(date_filter.to_string(), "created:2021-06-02");
    }

    #[test]
    fn test_label_filter_any_display() {
        let label_filter = LabelFilter::Any(["kind/bug", "area/bake"]);
        assert_eq!(label_filter.to_string(), r#"label:"kind/bug","area/bake""#);
    }

    #[test]
    fn test_label_filter_all_display() {
        let label_filter = LabelFilter::All(["kind/bug", "area/bake"]);
        assert_eq!(
            label_filter.to_string(),
            r#"label:"kind/bug" label:"area/bake""#
        );
    }

    #[test]
    fn test_label_filter_all_1_display() {
        let label_filter = LabelFilter::All(["kind/bug"]);
        assert_eq!(label_filter.to_string(), r#"label:"kind/bug""#);
    }
}
