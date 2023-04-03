use super::*;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Eq, PartialEq, Debug)]
pub struct ParseError(String);

impl ParseError {
    fn new<T>(message: T) -> Self
    where
        T: Display,
    {
        Self(message.to_string())
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "parse error: {}", self.0)
    }
}

impl VSpan {
    fn at(x: VInt) -> Self {
        VSpan::Point(x)
    }

    fn between(from: VInt, to: VInt) -> Self {
        assert!(from <= to);

        VSpan::Line(from, to)
    }

    /// merge in other, which must be touching
    fn merge(&mut self, other: Self) {
        assert!(self.touches(&other));

        use VSpan::*;
        match (&self, &other) {
            (Point(x), Point(y)) if x == y => (),
            _ => {
                *self = Line(
                    cmp::min(self.lower(), other.lower()),
                    cmp::max(self.upper(), other.upper()),
                )
            }
        }
    }

    /// whether other touches this, where a distance of 1 counts as touching
    fn touches(&self, other: &Self) -> bool {
        !(self.upper() + 1 < other.lower() || self.lower() > other.upper() + 1)
    }
}

/// trim whitespace and trailing a, b suffix
fn sanitize_verse(s: &str) -> &str {
    s.trim().trim_end_matches(|c| c == 'a' || c == 'b')
}

impl FromStr for VSpan {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((s1, s2)) => match (
                sanitize_verse(s1).parse::<VInt>(),
                sanitize_verse(s2).parse::<VInt>(),
            ) {
                (Ok(v1), Ok(v2)) => Ok(VSpan::between(v1, v2)),
                (Err(e1), Err(e2)) => Err(ParseError(format!(
                    "Verses::from_str error: {}, {}",
                    e1, e2
                ))),
                (Err(e1), _) => Err(ParseError::new(e1)),
                (_, Err(e2)) => Err(ParseError::new(e2)),
            },
            None => match sanitize_verse(s).parse::<VInt>() {
                Ok(v) => Ok(VSpan::at(v)),
                Err(e) => Err(ParseError::new(e)),
            },
        }
    }
}

impl VSpans {
    fn new() -> Self {
        Self(Vec::new())
    }

    /// determine leftmost item from i-1 and i
    fn leftmost_touching(&self, i: usize, item: &VSpan) -> Option<usize> {
        let touching_left = i > 0 && self.0[i - 1].touches(item);
        let touching_this = i < self.0.len() && self.0[i].touches(item);

        match (touching_left, touching_this) {
            (false, false) => None,
            (true, _) => Some(i - 1),
            (false, true) => Some(i),
        }
    }

    fn insert(&mut self, item: VSpan) {
        match self.0.binary_search(&item) {
            Ok(i) => {
                // repeated insert, ignore
                assert!(item == self.0[i]);
            }
            Err(i) => {
                match self.leftmost_touching(i, &item) {
                    None => self.0.insert(i, item),
                    Some(j) => {
                        self.0[j].merge(item);

                        // coalesce right until no more
                        while self.0.len() > j + 1 && self.0[j].touches(&self.0[j + 1]) {
                            let next = self.0.remove(j + 1);
                            self.0[j].merge(next);
                        }
                    }
                }
            }
        }
    }

    fn merge(&mut self, other: Self) {
        for item in other.0 {
            self.insert(item);
        }
    }
}

impl FromIterator<VSpan> for VSpans {
    fn from_iter<I: IntoIterator<Item = VSpan>>(iter: I) -> Self {
        let mut spans = Self::new();

        for s in iter {
            spans.insert(s);
        }

        spans
    }
}

impl FromStr for Chapter {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CInt::from_str(s).map(Self)
    }
}

impl ChaptersVerses {
    fn new(item: ChapterVerses) -> Self {
        Self(vec![item])
    }

    fn insert(&mut self, item: ChapterVerses) {
        match self.0.binary_search_by_key(&item.chapter, |cv| cv.chapter) {
            Ok(i) => self.0[i].verses.merge(item.verses),
            Err(i) => self.0.insert(i, item),
        }
    }
}

impl References {
    fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn _get(&self, book: &'static str) -> Option<&ChaptersVerses> {
        self.0.get(book)
    }

    fn insert(&mut self, book: &'static str, cv: ChapterVerses) {
        match self.0.get_mut(book) {
            Some(entry) => entry.insert(cv),
            None => {
                self.0.insert(book, ChaptersVerses::new(cv));
            }
        }
    }

    /// non-consuming iterator
    pub fn _iter(&self) -> std::collections::hash_map::Iter<&'static str, ChaptersVerses> {
        self.0.iter()
    }
}

/// get verses from the text, and return in order
fn verses(text: &str) -> VSpans {
    fn vspan_from_str(s: &str) -> Option<VSpan> {
        VSpan::from_str(s).ok()
    }

    text.split(',')
        .filter_map(vspan_from_str)
        .collect::<VSpans>()
}

pub fn references(text: &str) -> (References, Vec<String>) {
    lazy_static! {
        // 1. book chapter, which we use for later context
        // 2. book chapter:verses, which we extract, and store the context
        // 3. bare verse, which we extract using the stored context
        // 4. book verse
        static ref REFERENCE_RE: Regex =
            //           (bare verse          )(  prefix     book                  chapter verses)
            Regex::new(r"(\bv([\d:,\s-]+)[ab]?)|(([1-3]?)\s*([A-Z][[:alpha:]]+)\s*(\d{1,3}\b)?\s*([:v](\d[abv\d:,\s-]*))?)").unwrap();
    }

    let mut references = References::new();
    let mut warnings = Vec::new();

    let mut chapter_context: Option<ChapterContext> = None;

    for cap in REFERENCE_RE.captures_iter(text) {
        let fields = cap
            .iter()
            .map(|m_o| m_o.map(|m| m.as_str()))
            .collect::<Vec<Option<&str>>>();

        let book = book(fields[4], fields[5]);
        let chapter_str = fields[6];
        if let Some(book) = book {
            let chapter = chapter_str.map(|s| s.parse::<Chapter>().unwrap());

            if chapter.is_some() || is_single_chapter_book(book) {
                chapter_context = Some(ChapterContext { book, chapter });
            }
        }

        let vspans = match (fields[2], fields[8]) {
            (Some(_), Some(_)) => panic!("not possible to have both verse alternatives"),
            (Some(v), None) => verses(v),
            (None, Some(v)) => verses(v),
            (None, None) => VSpans::new(),
        };

        match chapter_context {
            Some(ctx) => {
                if ctx.chapter.is_some() || !vspans.is_empty() {
                    let cv = ChapterVerses::new(ctx.chapter, vspans);
                    // useful for generating test data
                    // println!(
                    //     "{} -> {} {}: {:?}",
                    //     fields[0].unwrap_or(" "),
                    //     &ctx.book,
                    //     &cv,
                    //     &fields
                    // );
                    references.insert(ctx.book, cv);
                }
            }
            None => {
                if !vspans.is_empty() {
                    warnings.push(format!("missing context for '{}'", fields[0].unwrap_or("")));
                }
            }
        }
    }

    (references, warnings)
}

mod tests;
