#![cfg(test)]

use super::*;
use test_case::test_case;

#[test]
fn test_verses_from_str() {
    assert_eq!(VSpan::from_str("7"), Ok(VSpan::at(7)));
    assert_eq!(VSpan::from_str("3-5"), Ok(VSpan::between(3, 5)));
    assert_eq!(VSpan::from_str(" 8  "), Ok(VSpan::at(8)));
    assert_eq!(VSpan::from_str(" 13   - 17 "), Ok(VSpan::between(13, 17)));
    assert!(VSpan::from_str("abc").is_err());
}

// helper for VSpan creation for tests
fn vspan(s: &str) -> VSpan {
    VSpan::from_str(s).unwrap()
}

#[test]
fn test_vspans_from_iter() {
    let result = VSpans::from_iter(vec![vspan("9"), vspan("1-7")]);
    let expected = VSpans(vec![vspan("1-7"), vspan("9")]);
    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_maintains_order() {
    let mut result = VSpans::new();

    result.insert(vspan("4"));
    result.insert(vspan("1-2"));
    result.insert(vspan("4"));
    result.insert(vspan("1-2"));

    let expected = VSpans(vec![vspan("1-2"), vspan("4")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_coalesces_left() {
    let mut result = VSpans::new();

    result.insert(vspan("2"));
    result.insert(vspan("3"));

    let expected = VSpans(vec![vspan("2-3")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_coalesces_right() {
    let mut result = VSpans::new();

    result.insert(vspan("3"));
    result.insert(vspan("1-2"));

    let expected = VSpans(vec![vspan("1-3")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_coalesces_both() {
    let mut result = VSpans::new();

    result.insert(VSpan::at(1));
    result.insert(VSpan::at(20));
    result.insert(VSpan::at(13));
    result.insert(VSpan::at(10));
    result.insert(VSpan::between(11, 12));

    let expected = VSpans(vec![vspan("1"), vspan("10-13"), vspan("20")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_insert_coalesces_both_2() {
    let mut result = VSpans::new();

    result.insert(vspan("4"));
    result.insert(vspan("9"));
    result.insert(vspan("2"));
    result.insert(vspan("1-7"));

    let expected = VSpans(vec![vspan("1-7"), vspan("9")]);

    assert_eq!(result, expected);
}

#[test]
fn test_vspans_merge() {
    let mut s0 = VSpans::from_iter(vec![vspan("9"), vspan("1-7")]);
    let s1 = VSpans::from_iter(vec![
        VSpan::at(1),
        VSpan::at(2),
        VSpan::at(8),
        VSpan::between(11, 12),
    ]);

    s0.merge(s1);
    let expected = VSpans(vec![vspan("1-9"), vspan("11-12")]);
    assert_eq!(s0, expected);
}

#[test]
fn test_verses() {
    assert_eq!(
        verses("4, 9, 2, 1-7"),
        VSpans(vec![vspan("1-7"), vspan("9")])
    );
}

#[test]
fn test_verses_with_letter_suffix() {
    assert_eq!(verses("4b, 9a"), VSpans(vec![vspan("4"), vspan("9")]));
}

#[test]
fn test_chapters_verses_insert() {
    let mut cv = ChaptersVerses::new(ChapterVerses {
        chapter: Some(Chapter(1)),
        verses: verses("1-3"),
    });

    cv.insert(ChapterVerses {
        chapter: Some(Chapter(2)),
        verses: verses("4"),
    });
    assert_eq!(
        cv,
        ChaptersVerses(vec![
            ChapterVerses {
                chapter: Some(Chapter(1)),
                verses: verses("1-3"),
            },
            ChapterVerses {
                chapter: Some(Chapter(2)),
                verses: verses("4"),
            }
        ])
    );

    cv.insert(ChapterVerses {
        chapter: Some(Chapter(1)),
        verses: verses("4"),
    });
    assert_eq!(
        cv,
        ChaptersVerses(vec![
            ChapterVerses {
                chapter: Some(Chapter(1)),
                verses: verses("1-4"),
            },
            ChapterVerses {
                chapter: Some(Chapter(2)),
                verses: verses("4"),
            }
        ])
    );

    cv.insert(ChapterVerses {
        chapter: Some(Chapter(2)),
        verses: verses("6"),
    });
    assert_eq!(
        cv,
        ChaptersVerses(vec![
            ChapterVerses {
                chapter: Some(Chapter(1)),
                verses: verses("1-4"),
            },
            ChapterVerses {
                chapter: Some(Chapter(2)),
                verses: verses("4, 6"),
            }
        ])
    );
}

#[test_case("1 Chronicles 28:9", "1 Chronicles 28:9")]
#[test_case("1 Cor 1:2", "1 Corinthians 1:2")]
#[test_case("1 Cor 4:17", "1 Corinthians 4:17")]
#[test_case("1 Corinthians 10:13", "1 Corinthians 10:13")]
#[test_case("1 Corinthians 1:27-31", "1 Corinthians 1:27-31")]
#[test_case("1 Corinthians 13:12b", "1 Corinthians 13:12")]
#[test_case("1 Corinthians 13", "1 Corinthians 13")]
#[test_case("1 Corinthians 3:15", "1 Corinthians 3:15")]
#[test_case("1 Corinthians 6:19-20", "1 Corinthians 6:19-20")]
#[test_case("1 Corinthians 9:24-25", "1 Corinthians 9:24-25")]
#[test_case("1 Jn 4:18", "1 John 4:18")]
#[test_case("1 Jn 4:9-10", "1 John 4:9-10")]
#[test_case("1 John 1:9", "1 John 1:9")]
#[test_case("1 John 3:16", "1 John 3:16")]
#[test_case("1 John 4:19", "1 John 4:19")]
#[test_case("1 Kings 18", "1 Kings 18")]
#[test_case("1 Kings 19:10, 14", "1 Kings 19:10,14")]
#[test_case("1 Kings 19:15-18", "1 Kings 19:15-18")]
#[test_case("1 Kings 19:19, 21", "1 Kings 19:19,21")]
#[test_case("1 Kings 19", "1 Kings 19")]
#[test_case("1 Kings 19:2-3", "1 Kings 19:2-3")]
#[test_case("1 Kings 19:4", "1 Kings 19:4")]
#[test_case("1 Kings 19:5-8", "1 Kings 19:5-8")]
#[test_case("1 Kings 19:9", "1 Kings 19:9")]
#[test_case("1 Peter 1:1-2", "1 Peter 1:1-2")]
#[test_case("1 Peter 1:6-7", "1 Peter 1:6-7")]
#[test_case("1 Peter 1:8-9", "1 Peter 1:8-9")]
#[test_case("1 Samuel 13:14", "1 Samuel 13:14")]
#[test_case("1 Thes 1:1", "1 Thessalonians 1:1")]
#[test_case("1 Thessalonians 1:2-3, 6b", "1 Thessalonians 1:2-3,6")]
#[test_case("1 Thessalonians 4:1", "1 Thessalonians 4:1")]
#[test_case("1 Thessalonians 5:17", "1 Thessalonians 5:17")]
#[test_case("1 Thessalonians 5:24", "1 Thessalonians 5:24")]
#[test_case("1 Thessalonians 5:8", "1 Thessalonians 5:8")]
#[test_case("1 Tim 1:15", "1 Timothy 1:15")]
#[test_case("1 Tim 1:2", "1 Timothy 1:2")]
#[test_case("1 Timothy 2:12", "1 Timothy 2:12")]
#[test_case("1 Timothy 2:13-14", "1 Timothy 2:13-14")]
#[test_case("1 Timothy 6:11", "1 Timothy 6:11")]
#[test_case("2 Cor 12:7", "2 Corinthians 12:7")]
#[test_case("2 Cor 12:7-9", "2 Corinthians 12:7-9")]
#[test_case("2 Cor 5:17,", "2 Corinthians 5:17")]
#[test_case("2 Corinthians 11:24-28", "2 Corinthians 11:24-28")]
#[test_case("2 Corinthians 12:9, 10", "2 Corinthians 12:9-10")]
#[test_case("2 Corinthians 1:3-4", "2 Corinthians 1:3-4")]
#[test_case("2 Corinthians 5:17", "2 Corinthians 5:17")]
#[test_case("2 Corinthians 5:7", "2 Corinthians 5:7")]
#[test_case("2 Corinthians 7:10", "2 Corinthians 7:10")]
#[test_case("2 Corinthians 7:8", "2 Corinthians 7:8")]
#[test_case("2 Corinthians 7:9", "2 Corinthians 7:9")]
#[test_case("2 Kings 2:1-15", "2 Kings 2:1-15")]
#[test_case("2 Kings 5", "2 Kings 5")]
#[test_case("2 Kings 6:4-7", "2 Kings 6:4-7")]
#[test_case("2 Peter 1:1", "2 Peter 1:1")]
#[test_case("2 Samuel 11", "2 Samuel 11")]
#[test_case("2 Tim 1:2", "2 Timothy 1:2")]
#[test_case("Acts 22:3,", "Acts 22:3")]
#[test_case("Acts 28", "Acts 28")]
#[test_case("Col 1:2", "Colossians 1:2")]
#[test_case("Col 3:3", "Colossians 3:3")]
#[test_case("Colossians 1:5-6", "Colossians 1:5-6")]
#[test_case(" Colossians 1:9-11", "Colossians 1:9-11")]
#[test_case("Colossians 3:12", "Colossians 3:12")]
#[test_case("Daniel 1:8a", "Daniel 1:8")]
#[test_case("Daniel 3:16-18", "Daniel 3:16-18")]
#[test_case("Deuteronomy 3:26", "Deuteronomy 3:26")]
#[test_case("Deuteronomy 6:13", "Deuteronomy 6:13")]
#[test_case("Deuteronomy 6:16", "Deuteronomy 6:16")]
#[test_case("Deuteronomy 8:3", "Deuteronomy 8:3")]
#[test_case("Eph 1:1", "Ephesians 1:1")]
#[test_case("Eph 1:3", "Ephesians 1:3")]
#[test_case("Ephesians 1:13-14", "Ephesians 1:13-14")]
#[test_case("Ephesians 1:17", "Ephesians 1:17")]
#[test_case("Ephesians 1:18-20", "Ephesians 1:18-20")]
#[test_case("Ephesians 1:3", "Ephesians 1:3")]
#[test_case("Ephesians 1:4, 15-18", "Ephesians 1:4,15-18")]
#[test_case("Ephesians 1:4-5, 11-12", "Ephesians 1:4-5,11-12")]
#[test_case("Ephesians 1:7-9", "Ephesians 1:7-9")]
#[test_case(" Ephesians 1", "Ephesians 1")]
#[test_case("Ephesians 2:10", "Ephesians 2:10")]
#[test_case("Ephesians 2:21-22", "Ephesians 2:21-22")]
#[test_case("Ephesians 2:8-9", "Ephesians 2:8-9")]
#[test_case("Ephesians 3:20-21", "Ephesians 3:20-21")]
#[test_case("Ephesians 4:1-3", "Ephesians 4:1-3")]
#[test_case("Ephesians 4:1", "Ephesians 4:1")]
#[test_case("Ephesians 5:18b", "Ephesians 5:18")]
#[test_case("Ephesians 6:10-11", "Ephesians 6:10-11")]
#[test_case("Ephesians 6:12", "Ephesians 6:12")]
#[test_case("Ephesians 6:13, 15", "Ephesians 6:13,15")]
#[test_case("Ephesians 6:13, 17a", "Ephesians 6:13,17")]
#[test_case("Ephesians 6:14a", "Ephesians 6:14")]
#[test_case("Ephesians 6:14", "Ephesians 6:14")]
#[test_case("Ephesians 6:16", "Ephesians 6:16")]
#[test_case("Ephesians 6:17", "Ephesians 6:17")]
#[test_case("Ephesians 6:18", "Ephesians 6:18")]
#[test_case("Exodus 2:23", "Exodus 2:23")]
#[test_case("Exodus 2:24-25", "Exodus 2:24-25")]
#[test_case("Exodus 34:5-8", "Exodus 34:5-8")]
#[test_case("Exodus 34:6-7", "Exodus 34:6-7")]
#[test_case("Exodus 3:7-8", "Exodus 3:7-8")]
#[test_case("Exodus 4:11-13", "Exodus 4:11-13")]
#[test_case("Galatians 5:16-18", "Galatians 5:16-18")]
#[test_case("Galatians 5:17, 19-23", "Galatians 5:17,19-23")]
#[test_case(" Galatians 5:18", "Galatians 5:18")]
#[test_case("Galatians 5:22-23", "Galatians 5:22-23")]
#[test_case("Galatians 5:22-25", "Galatians 5:22-25")]
#[test_case("Galatians 5:22", "Galatians 5:22")]
#[test_case(" Gen 17:1", "Genesis 17:1")]
#[test_case("Genesis 12:10-20", "Genesis 12:10-20")]
#[test_case("Genesis 1:31", "Genesis 1:31")]
#[test_case("Genesis 16", "Genesis 16")]
#[test_case("Genesis 17:1", "Genesis 17:1")]
#[test_case("Genesis 25", "Genesis 25")]
#[test_case("Genesis 3:14a", "Genesis 3:14")]
#[test_case("Genesis 3:17", "Genesis 3:17")]
#[test_case("Genesis 35:9-12", "Genesis 35:9-12")]
#[test_case("Genesis 47:27", "Genesis 47:27")]
#[test_case("Genesis 6:5-7", "Genesis 6:5-7")]
#[test_case("Genesis 6:8", "Genesis 6:8")]
#[test_case("Genesis 6:9", "Genesis 6:9")]
#[test_case("Genesis 9:20-21", "Genesis 9:20-21")]
#[test_case("Habakkuk 1:2", "Habakkuk 1:2")]
#[test_case("Habakkuk 1:5", "Habakkuk 1:5")]
#[test_case("Heb 10:23", "Hebrews 10:23")]
#[test_case("Heb 4:15", "Hebrews 4:15")]
#[test_case("Hebrews 10:23", "Hebrews 10:23")]
#[test_case("Hebrews 11:1-2", "Hebrews 11:1-2")]
#[test_case("Hebrews 12:1-3", "Hebrews 12:1-3")]
#[test_case("Hebrews 12:1", "Hebrews 12:1")]
#[test_case("Hebrews 13:8", "Hebrews 13:8")]
#[test_case("Isaiah 42:3", "Isaiah 42:3")]
#[test_case("Isaiah 43:16, 18-19", "Isaiah 43:16,18-19")]
#[test_case("James 2:18", "James 2:18")]
#[test_case("James 5:14-15", "James 5:14-15")]
#[test_case("Jer 29:11", "Jeremiah 29:11")]
#[test_case("Jer 32:27", "Jeremiah 32:27")]
#[test_case("Jeremiah 17:7-8", "Jeremiah 17:7-8")]
#[test_case("Jeremiah 29:13-14a", "Jeremiah 29:13-14")]
#[test_case("Jeremiah 32:26-27", "Jeremiah 32:26-27")]
#[test_case("Jeremiah 32:27", "Jeremiah 32:27")]
#[test_case("Jeremiah 9:24", "Jeremiah 9:24")]
#[test_case("Job 1:8-11", "Job 1:8-11")]
#[test_case(" Job 38", "Job 38")]
#[test_case("John 10:10b", "John 10:10")]
#[test_case("John 10:10", "John 10:10")]
#[test_case("John 11:1, 3, 6, 17, 21", "John 11:1,3,6,17,21")]
#[test_case("John 11:5, 36", "John 11:5,36")]
#[test_case("John 11:5-6", "John 11:5-6")]
#[test_case("John 13:23", "John 13:23")]
#[test_case("John 13:34-35", "John 13:34-35")]
#[test_case("John 14:13-14", "John 14:13-14")]
#[test_case("John 14:14", "John 14:14")]
#[test_case("John 14:27", "John 14:27")]
#[test_case(" John 14", "John 14")]
#[test_case("John 19:26", "John 19:26")]
#[test_case("John 19:30", "John 19:30")]
#[test_case("John 20:1-2", "John 20:1-2")]
#[test_case("John 21:20", "John 21:20")]
#[test_case("John 3:16", "John 3:16")]
#[test_case("John 6:28-29", "John 6:28-29")]
#[test_case("John 8:31-32", "John 8:31-32")]
#[test_case("John 8:32", "John 8:32")]
#[test_case("John 8:44b", "John 8:44")]
#[test_case("John 8:51", "John 8:51")]
#[test_case("Josh 23:14", "Joshua 23:14")]
#[test_case("Joshua 23:14", "Joshua 23:14")]
#[test_case("Joshua 6:2", "Joshua 6:2")]
#[test_case("Jude v24-25", "Jude v24-25")]
#[test_case("Lamentations 1:3, 5, 8, 12", "Lamentations 1:3,5,8,12")]
#[test_case("Lamentations 3:21-23", "Lamentations 3:21-23")]
#[test_case("Lamentations 3:21-26", "Lamentations 3:21-26")]
#[test_case("Lamentations 3:31-33", "Lamentations 3:31-33")]
#[test_case("Luke 1:26-56", "Luke 1:26-56")]
#[test_case("Luke 1:30", "Luke 1:30")]
#[test_case("Luke 1:38", "Luke 1:38")]
#[test_case("Luke 15:10", "Luke 15:10")]
#[test_case("Luke 1:57-58", "Luke 1:57-58")]
#[test_case("Luke 17:3-4", "Luke 17:3-4")]
#[test_case("Luke 18:10-14", "Luke 18:10-14")]
#[test_case("Luke 18:17", "Luke 18:17")]
#[test_case("Luke 23:33-34a", "Luke 23:33-34")]
#[test_case("Luke 4:1-13", "Luke 4:1-13")]
#[test_case(r#"
{{< quote source="Luke 22:55-60" >}}
And when they had kindled a fire in the middle of the courtyard and sat down together, Peter sat down among them. Then a servant girl, seeing him as he sat in the light and looking closely at him, said, “This man also was with him.” But he denied it, saying, “Woman, I do not know him.” And a little later someone else saw him and said, “You also are one of them.” But Peter said, “Man, I am not.” And after an interval of about an hour still another insisted, saying, “Certainly this man also was with him, for he too is a Galilean.” But Peter said, “Man, I do not know what you are talking about.” And immediately, while he was still speaking, the rooster crowed.
{{< /quote >}}

And then what?

{{< quote source="v61a" >}}
And the Lord turned and looked at Peter.
{{< /quote >}}

What would have been in that look from Jesus? Disappointment surely, and sadness⸺sadness beyond imagining⸺but also love, such love. Steadfast love. Irrefutable love. The love that would send Jesus to the cross, in obedience to his Father, love even unto death. Love for Peter.

{{< quote source="v61b-62" >}}
And Peter remembered the saying of the Lord, how he had said to him, “Before the rooster crows today, you will deny me three times.” And he went out and wept bitterly.
{{< /quote >}}
"#, "Luke 22:55-62")]
#[test_case("Malachi 3:6-7", "Malachi 3:6-7")]
#[test_case("Malachi 3:6a", "Malachi 3:6")]
#[test_case("Mark 11:12-14, 20", "Mark 11:12-14,20")]
#[test_case("Mark 11:23", "Mark 11:23")]
#[test_case("Mark 11", "Mark 11")]
#[test_case("Mark 12:28b", "Mark 12:28")]
#[test_case("Mark 14:61-62", "Mark 14:61-62")]
#[test_case("Mark 14:61", "Mark 14:61")]
#[test_case("Mark 15:22, 24", "Mark 15:22,24")]
#[test_case("Mark 15:34", "Mark 15:34")]
#[test_case("Matthew 11:28-30", "Matthew 11:28-30")]
#[test_case("Matthew 1:18-23", "Matthew 1:18-23")]
#[test_case("Matthew 1:19", "Matthew 1:19")]
#[test_case("Matthew 12:6-8", "Matthew 12:6-8")]
#[test_case("Matthew 19:30", "Matthew 19:30")]
#[test_case("Matthew 19:8-9", "Matthew 19:8-9")]
#[test_case("Matthew 2:11", "Matthew 2:11")]
#[test_case("Matthew 21:22", "Matthew 21:22")]
#[test_case("Matthew 2:1-2, 9-12", "Matthew 2:1-2,9-12")]
#[test_case("Matthew 22:36-40", "Matthew 22:36-40")]
#[test_case("Matthew 25:23", "Matthew 25:23")]
#[test_case("Matthew 25:35-36, 40", "Matthew 25:35-36,40")]
#[test_case("Matthew 5:17", "Matthew 5:17")]
#[test_case("Matthew 5:20", "Matthew 5:20")]
#[test_case("Matthew 5:8", "Matthew 5:8")]
#[test_case("Matthew 6:12, 14-15", "Matthew 6:12,14-15")]
#[test_case("Matthew 6:33", "Matthew 6:33")]
#[test_case("Matthew 7:7, 11", "Matthew 7:7,11")]
#[test_case("Numbers 12:3", "Numbers 12:3")]
#[test_case("Numbers 20:12", "Numbers 20:12")]
#[test_case("Numbers 23:19", "Numbers 23:19")]
#[test_case("Phil 1:1", "Philippians 1:1")]
#[test_case(" Phil 2:20-22", "Philippians 2:20-22")]
#[test_case("Phil 3:4-6", "Philippians 3:4-6")]
#[test_case("Philemon 1:1", "Philemon 1:1")]
#[test_case("Philippians 2:13", "Philippians 2:13")]
#[test_case("Philippians 2:5-11", "Philippians 2:5-11")]
#[test_case("Philippians 3:4-6", "Philippians 3:4-6")]
#[test_case(" Philippians 4:10-20", "Philippians 4:10-20")]
#[test_case("Philippians 4:11b", "Philippians 4:11")]
#[test_case("Philippians 4:13", "Philippians 4:13")]
#[test_case("Philippians 4:15-18", "Philippians 4:15-18")]
#[test_case("Philippians 4:19", "Philippians 4:19")]
#[test_case("Philippians 4:4-5", "Philippians 4:4-5")]
#[test_case("Philippians 4:4", "Philippians 4:4")]
#[test_case("Philippians 4:8", "Philippians 4:8")]
#[test_case("Proverbs 16:9", "Proverbs 16:9")]
#[test_case("Proverbs 4:23", "Proverbs 4:23")]
#[test_case("Proverbs 7:23", "Proverbs 7:23")]
#[test_case(" Proverbs 7", "Proverbs 7")]
#[test_case("Proverbs 9:10", "Proverbs 9:10")]
#[test_case(" Ps 139", "Psalms 139")]
#[test_case("Psalm 119:18", "Psalms 119:18")]
#[test_case("Psalm 119:9-16", "Psalms 119:9-16")]
#[test_case(" Psalm 119", "Psalms 119")]
#[test_case("Psalm 13:1-2", "Psalms 13:1-2")]
#[test_case(" Psalm 133:1", "Psalms 133:1")]
#[test_case(" Psalm 135", "Psalms 135")]
#[test_case("Psalm 139:13-16", "Psalms 139:13-16")]
#[test_case("Psalm 139:1-4, 13-14a", "Psalms 139:1-4,13-14")]
#[test_case(" Psalm 139", "Psalms 139")]
#[test_case(" Psalm 145", "Psalms 145")]
#[test_case("Psalm 147:11", "Psalms 147:11")]
#[test_case("Psalm 147:1", "Psalms 147:1")]
#[test_case("Psalm 147:2-6", "Psalms 147:2-6")]
#[test_case(" Psalm 16:6", "Psalms 16:6")]
#[test_case("Psalm 18:46-49", "Psalms 18:46-49")]
#[test_case("Psalm 19", "Psalms 19")]
#[test_case("Psalm 23:1-3", "Psalms 23:1-3")]
#[test_case("Psalm 23:4", "Psalms 23:4")]
#[test_case(" Psalm 23", "Psalms 23")]
#[test_case("Psalm 25:10", "Psalms 25:10")]
#[test_case("Psalm 25:21", "Psalms 25:21")]
#[test_case("Psalm 25:8-9", "Psalms 25:8-9")]
#[test_case(" Psalm 25", "Psalms 25")]
#[test_case("Psalm 27:13", "Psalms 27:13")]
#[test_case("Psalm 27", "Psalms 27")]
#[test_case("Psalm 33:4", "Psalms 33:4")]
#[test_case("Psalm 3:3", "Psalms 3:3")]
#[test_case("Psalm 34:5", "Psalms 34:5")]
#[test_case("Psalm 34:8", "Psalms 34:8")]
#[test_case("Psalm 36:5", "Psalms 36:5")]
#[test_case("Psalm 37:4", "Psalms 37:4")]
#[test_case(" Psalm 3", "Psalms 3")]
#[test_case("Psalm 46:2, 6", "Psalms 46:2,6")]
#[test_case("Psalm 46", "Psalms 46")]
#[test_case("Psalm 51:6a", "Psalms 51:6")]
#[test_case("Psalm 51:6", "Psalms 51:6")]
#[test_case("Psalm 69:30", "Psalms 69:30")]
#[test_case("Psalm 91:14", "Psalms 91:14")]
#[test_case(" Rev 20:12", "Revelation 20:12")]
#[test_case("Rev 21:3-4", "Revelation 21:3-4")]
#[test_case("Rev 4:11", "Revelation 4:11")]
#[test_case("Rev 4:3b-6a", "Revelation 4:3-6")]
#[test_case("Revelation 1:16", "Revelation 1:16")]
#[test_case("Revelation 12:10", "Revelation 12:10")]
#[test_case("Revelation 19:11", "Revelation 19:11")]
#[test_case("Revelation 21:1-2", "Revelation 21:1-2")]
#[test_case("Revelation 4:2", "Revelation 4:2")]
#[test_case("Revelation 4:3a", "Revelation 4:3")]
#[test_case("Revelation 4:3b", "Revelation 4:3")]
#[test_case("Revelation 4:6b", "Revelation 4:6")]
#[test_case(" Revelation 4:8,", "Revelation 4:8")]
#[test_case("Revelation 5:12", "Revelation 5:12")]
#[test_case("Revelation 5:13-14", "Revelation 5:13-14")]
#[test_case(" Revelation 5:9,", "Revelation 5:9")]
#[test_case("Rom 1:7", "Romans 1:7")]
#[test_case("Rom 4:20", "Romans 4:20")]
#[test_case(" Rom 8:1-4, 31-39", "Romans 8:1-4,31-39")]
#[test_case("Romans 10:1", "Romans 10:1")]
#[test_case("Romans 11:33, 36", "Romans 11:33,36")]
#[test_case("Romans 1:17", "Romans 1:17")]
#[test_case("Romans 12:1", "Romans 12:1")]
#[test_case("Romans 12:2", "Romans 12:2")]
#[test_case("Romans 13:10b", "Romans 13:10")]
#[test_case("Romans 15:9", "Romans 15:9")]
#[test_case("Romans 3:21-22, 27", "Romans 3:21-22,27")]
#[test_case("Romans 3:22-24", "Romans 3:22-24")]
#[test_case("Romans 3:22b", "Romans 3:22")]
#[test_case("Romans 3:22", "Romans 3:22")]
#[test_case("Romans 4:18", "Romans 4:18")]
#[test_case("Romans 4:20-21", "Romans 4:20-21")]
#[test_case(" Romans 4", "Romans 4")]
#[test_case("Romans 5:8", "Romans 5:8")]
#[test_case("Romans 6:13-14", "Romans 6:13-14")]
#[test_case("Romans 6:14-15", "Romans 6:14-15")]
#[test_case("Romans 6:1-4", "Romans 6:1-4")]
#[test_case("Romans 6:17-18", "Romans 6:17-18")]
#[test_case("Romans 6:18", "Romans 6:18")]
#[test_case("Romans 7:24-25a", "Romans 7:24-25")]
#[test_case("Romans 8:13-14", "Romans 8:13-14")]
#[test_case("Romans 8:14", "Romans 8:14")]
#[test_case("Romans 8:18-21", "Romans 8:18-21")]
#[test_case(" Romans 8:18-27,", "Romans 8:18-27")]
#[test_case("Romans 8:22-25", "Romans 8:22-25")]
#[test_case("Romans 8:26", "Romans 8:26")]
#[test_case(" Romans 8:28a", "Romans 8:28")]
#[test_case("Romans 8:28", "Romans 8:28")]
#[test_case("Romans 8:32", "Romans 8:32")]
#[test_case("Romans 8:33-34a", "Romans 8:33-34")]
#[test_case("Romans 9:14-15", "Romans 9:14-15")]
#[test_case("Romans 9:21-23", "Romans 9:21-23")]
#[test_case("Romans 9:2-3", "Romans 9:2-3")]
#[test_case("Romans 9", "Romans 9")]
#[test_case("Titus 3:4-5", "Titus 3:4-5")]
#[test_case("Zechariah 9:9", "Zechariah 9:9")]
// TODO context
// TODO (" Mitre 10", "2 Kings 2")]
// TODO ? (" Society 22", "1 Kings 19"),
// ("v12", "1 John 4:12"),
// ("v1,", "2 Kings 5:1"),
// ("v1", "2 Peter 1:1"),
// ("v12", "Psalms 25:12"),
// ("v13", "Psalms 27:13"),
// ("v14", "Psalms 25:14"),
// ("v14", "Psalms 27:14"),
// ("v17", "1 John 4:17"),
// ("v18", "2 Kings 5:18"),
// ("v1", "Psalms 23:1"),
// ("v1", "Psalms 27:1"),
// ("v21", "Psalms 145:21"),
// ("v2", "2 Kings 5:2"),
// ("v24", "Luke 15:24"),
// ("v2", "Psalms 145:2"),
// ("v30, 32", "Psalms 119:30,32"),
// ("v3", "2 Kings 5:3"),
// ("v32", "Psalms 119:32"),
// ("v3", "Ephesians 1:3"),
// ("v3", "Psalms 135:3"),
// ("v3", "Psalms 145:3"),
// ("v3", "Psalms 23:3"),
// ("v4", "Psalms 23:4"),
// ("v4", "Psalms 27:4"),
// ("v5", "2 Corinthians 1:5"),
// ("v5", "Psalms 145:5"),
// ("v5", "Psalms 23:5"),
// ("v5", "Psalms 27:5"),
// ("v6", "Psalms 46:6"),
// ("v7", "2 Kings 5:7"),
// ("v8-9, 13-14, 17-20", "Psalms 145:8-9,13-14,17-20"),
// ("v8", "Psalms 27:8"),
// ("v9, 10", "Psalms 46:9-10"),
// ("v9,", "Psalms 27:9"),
fn test_references(src: &str, expected: &str) {
    let (refs, warnings) = references("", src);
    assert!(warnings.is_empty(), "{}", src);
    assert!(refs.0.len() == 1, "{}", src);
    let book = *(refs.0.keys().next().unwrap());
    let cvs = &refs.0[book];

    let result = format!("{} {}", book, cvs);
    assert_eq!(result, expected, "{}", src);
}
