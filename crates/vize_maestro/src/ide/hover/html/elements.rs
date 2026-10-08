//! Borrowed documentation for the existing admitted HTML DOM map.
//! Descriptions are original Vize prose; reference links were verified against
//! MDN and the HTML Living Standard. No documentation is fetched at runtime.

pub(super) struct ElementDocs {
    pub(super) tag: &'static str,
    pub(super) description: &'static str,
    pub(super) mdn_url: &'static str,
    pub(super) standard_url: &'static str,
}

macro_rules! elements {
    ($(($tag:literal, $description:literal, $mdn:literal, $standard:literal)),* $(,)?) => {
        static ELEMENTS: &[ElementDocs] = &[$(ElementDocs {
            tag: $tag,
            description: $description,
            mdn_url: concat!("https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/", $mdn),
            standard_url: concat!("https://html.spec.whatwg.org/multipage/", $standard),
        }),*];
    };
}

// Keep sorted: cold lookups need at most seven binary-search comparisons.
elements! {
    ("a", "Creates a hyperlink when href is present; otherwise it marks a placeholder for a link.", "a", "text-level-semantics.html#the-a-element"),
    ("abbr", "Marks an abbreviation or acronym; its title can supply the expanded form.", "abbr", "text-level-semantics.html#the-abbr-element"),
    ("address", "Provides contact information for the nearest article or the document.", "address", "sections.html#the-address-element"),
    ("area", "Defines an image-map region, optionally linking to a resource through href.", "area", "image-maps.html#the-area-element"),
    ("article", "Groups self-contained content that can be distributed or reused independently.", "article", "sections.html#the-article-element"),
    ("aside", "Groups content related to, but separate from, the surrounding main content.", "aside", "sections.html#the-aside-element"),
    ("audio", "Embeds sound with optional playback controls and alternative sources.", "audio", "media.html#the-audio-element"),
    ("b", "Draws attention to text without adding importance or changing its voice.", "b", "text-level-semantics.html#the-b-element"),
    ("base", "Sets the base URL or default target for relative links in the document.", "base", "semantics.html#the-base-element"),
    ("bdi", "Isolates text so its writing direction does not affect surrounding text.", "bdi", "text-level-semantics.html#the-bdi-element"),
    ("bdo", "Overrides the writing direction of its text.", "bdo", "text-level-semantics.html#the-bdo-element"),
    ("blockquote", "Contains a section quoted from another source.", "blockquote", "grouping-content.html#the-blockquote-element"),
    ("body", "Contains the document content shown to the user.", "body", "sections.html#the-body-element"),
    ("br", "Inserts a line break within text.", "br", "text-level-semantics.html#the-br-element"),
    ("button", "Provides an interactive button that can trigger an action or submit a form.", "button", "form-elements.html#the-button-element"),
    ("canvas", "Provides a drawing surface for graphics rendered by scripts.", "canvas", "canvas.html#the-canvas-element"),
    ("caption", "Gives a table its title or explanatory caption.", "caption", "tables.html#the-caption-element"),
    ("cite", "Identifies the title of a creative work being referenced.", "cite", "text-level-semantics.html#the-cite-element"),
    ("code", "Marks a fragment of computer code.", "code", "text-level-semantics.html#the-code-element"),
    ("col", "Specifies properties shared by a column in a table.", "col", "tables.html#the-col-element"),
    ("colgroup", "Groups columns within a table.", "colgroup", "tables.html#the-colgroup-element"),
    ("data", "Associates human-readable content with a machine-readable value.", "data", "text-level-semantics.html#the-data-element"),
    ("datalist", "Supplies suggested values for a linked input control.", "datalist", "form-elements.html#the-datalist-element"),
    ("dd", "Provides a description or value for a term in a description list.", "dd", "grouping-content.html#the-dd-element"),
    ("del", "Marks content removed from a document.", "del", "edits.html#the-del-element"),
    ("details", "Creates a disclosure widget whose summary can reveal additional content.", "details", "interactive-elements.html#the-details-element"),
    ("dfn", "Marks the term being defined in its surrounding context.", "dfn", "text-level-semantics.html#the-dfn-element"),
    ("dialog", "Represents a dialog box or other interactive subwindow.", "dialog", "interactive-elements.html#the-dialog-element"),
    ("div", "Groups content without adding a specific semantic meaning.", "div", "grouping-content.html#the-div-element"),
    ("dl", "Groups terms and their associated descriptions or values.", "dl", "grouping-content.html#the-dl-element"),
    ("dt", "Provides a term or name in a description list.", "dt", "grouping-content.html#the-dt-element"),
    ("em", "Adds stress emphasis to text.", "em", "text-level-semantics.html#the-em-element"),
    ("embed", "Embeds external content such as a document or interactive resource.", "embed", "iframe-embed-object.html#the-embed-element"),
    ("fieldset", "Groups related form controls, optionally with a legend.", "fieldset", "form-elements.html#the-fieldset-element"),
    ("figcaption", "Provides a caption for the content of its parent figure.", "figcaption", "grouping-content.html#the-figcaption-element"),
    ("figure", "Groups self-contained content with an optional caption.", "figure", "grouping-content.html#the-figure-element"),
    ("footer", "Provides footer information for its nearest section or the document.", "footer", "sections.html#the-footer-element"),
    ("form", "Groups controls for collecting and submitting user input.", "form", "forms.html#the-form-element"),
    ("h1", "Provides a level-1 section heading; h1 is the highest heading level and h6 the lowest.", "Heading_Elements", "sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements"),
    ("h2", "Provides a level-2 section heading; h1 is the highest heading level and h6 the lowest.", "Heading_Elements", "sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements"),
    ("h3", "Provides a level-3 section heading; h1 is the highest heading level and h6 the lowest.", "Heading_Elements", "sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements"),
    ("h4", "Provides a level-4 section heading; h1 is the highest heading level and h6 the lowest.", "Heading_Elements", "sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements"),
    ("h5", "Provides a level-5 section heading; h1 is the highest heading level and h6 the lowest.", "Heading_Elements", "sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements"),
    ("h6", "Provides a level-6 section heading; h1 is the highest heading level and h6 the lowest.", "Heading_Elements", "sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements"),
    ("head", "Contains document metadata such as its title, styles, and resource links.", "head", "semantics.html#the-head-element"),
    ("header", "Provides introductory content or navigation for its section or the document.", "header", "sections.html#the-header-element"),
    ("hgroup", "Groups a heading with related secondary content such as a subtitle.", "hgroup", "sections.html#the-hgroup-element"),
    ("hr", "Marks a thematic break between sections of content.", "hr", "grouping-content.html#the-hr-element"),
    ("html", "Forms the root element of an HTML document.", "html", "semantics.html#the-html-element"),
    ("i", "Marks text in an alternate voice or mood, such as a technical term.", "i", "text-level-semantics.html#the-i-element"),
    ("iframe", "Embeds another document in a nested browsing context.", "iframe", "iframe-embed-object.html#the-iframe-element"),
    ("img", "Embeds an image with alternative text supplied by its alt attribute.", "img", "embedded-content.html#the-img-element"),
    ("input", "Provides a form control whose behavior depends on its type.", "input", "input.html#the-input-element"),
    ("ins", "Marks content added to a document.", "ins", "edits.html#the-ins-element"),
    ("kbd", "Marks user input, such as keyboard commands.", "kbd", "text-level-semantics.html#the-kbd-element"),
    ("label", "Provides a caption associated with a form control.", "label", "forms.html#the-label-element"),
    ("legend", "Provides the caption for its parent fieldset.", "legend", "form-elements.html#the-legend-element"),
    ("li", "Represents one item in a list.", "li", "grouping-content.html#the-li-element"),
    ("link", "Declares a relationship to an external resource, such as a stylesheet.", "link", "semantics.html#the-link-element"),
    ("main", "Contains the dominant content of the document body.", "main", "grouping-content.html#the-main-element"),
    ("map", "Defines an image map containing clickable regions.", "map", "image-maps.html#the-map-element"),
    ("mark", "Highlights text relevant to the current context.", "mark", "text-level-semantics.html#the-mark-element"),
    ("menu", "Represents a list of items, often containing user commands.", "menu", "grouping-content.html#the-menu-element"),
    ("meta", "Supplies document metadata not expressed by another metadata element.", "meta", "semantics.html#the-meta-element"),
    ("meter", "Displays a scalar measurement within a known range.", "meter", "form-elements.html#the-meter-element"),
    ("nav", "Groups links for navigating the document or related documents.", "nav", "sections.html#the-nav-element"),
    ("noscript", "Supplies alternative content when scripting is unavailable or disabled.", "noscript", "scripting.html#the-noscript-element"),
    ("object", "Embeds an external resource such as an image or nested document.", "object", "iframe-embed-object.html#the-object-element"),
    ("ol", "Groups list items whose order is meaningful.", "ol", "grouping-content.html#the-ol-element"),
    ("optgroup", "Groups related options within a select control.", "optgroup", "form-elements.html#the-optgroup-element"),
    ("option", "Defines a selectable item in a select or a suggestion in a datalist.", "option", "form-elements.html#the-option-element"),
    ("output", "Displays the result of a calculation or user action.", "output", "form-elements.html#the-output-element"),
    ("p", "Represents a paragraph.", "p", "grouping-content.html#the-p-element"),
    ("picture", "Offers alternative image sources for a contained img element.", "picture", "embedded-content.html#the-picture-element"),
    ("pre", "Preserves whitespace in a block of preformatted text.", "pre", "grouping-content.html#the-pre-element"),
    ("progress", "Displays how much of a task has been completed.", "progress", "form-elements.html#the-progress-element"),
    ("q", "Marks a short quotation within running text.", "q", "text-level-semantics.html#the-q-element"),
    ("rp", "Provides fallback parentheses around ruby annotations.", "rp", "text-level-semantics.html#the-rp-element"),
    ("rt", "Provides a pronunciation or other ruby annotation for nearby text.", "rt", "text-level-semantics.html#the-rt-element"),
    ("ruby", "Groups text with pronunciation or other small annotations.", "ruby", "text-level-semantics.html#the-ruby-element"),
    ("s", "Marks content that is no longer accurate or relevant.", "s", "text-level-semantics.html#the-s-element"),
    ("samp", "Marks sample output from a program or computing system.", "samp", "text-level-semantics.html#the-samp-element"),
    ("script", "Contains or references executable script or a data block.", "script", "scripting.html#the-script-element"),
    ("search", "Groups controls or content used to search or filter information.", "search", "grouping-content.html#the-search-element"),
    ("section", "Groups a thematic section of content, usually with a heading.", "section", "sections.html#the-section-element"),
    ("select", "Provides a control for selecting from a set of options.", "select", "form-elements.html#the-select-element"),
    ("small", "Marks side comments or fine print.", "small", "text-level-semantics.html#the-small-element"),
    ("source", "Provides an alternative media resource for audio, video, or picture.", "source", "embedded-content.html#the-source-element"),
    ("span", "Groups phrasing content without adding a specific semantic meaning.", "span", "text-level-semantics.html#the-span-element"),
    ("strong", "Marks text with strong importance, seriousness, or urgency.", "strong", "text-level-semantics.html#the-strong-element"),
    ("style", "Contains CSS rules for the document.", "style", "semantics.html#the-style-element"),
    ("sub", "Marks text as a subscript, such as a chemical formula index.", "sub", "text-level-semantics.html#the-sub-and-sup-elements"),
    ("summary", "Provides the visible label for a details disclosure widget.", "summary", "interactive-elements.html#the-summary-element"),
    ("sup", "Marks text as a superscript, such as an exponent.", "sup", "text-level-semantics.html#the-sub-and-sup-elements"),
    ("table", "Organizes data into rows and columns.", "table", "tables.html#the-table-element"),
    ("tbody", "Groups the body rows of a table.", "tbody", "tables.html#the-tbody-element"),
    ("td", "Provides a data cell in a table row.", "td", "tables.html#the-td-element"),
    ("textarea", "Provides a multiline plain-text input control.", "textarea", "form-elements.html#the-textarea-element"),
    ("tfoot", "Groups footer rows of a table, such as totals.", "tfoot", "tables.html#the-tfoot-element"),
    ("th", "Provides a header cell for a row or column in a table.", "th", "tables.html#the-th-element"),
    ("thead", "Groups the header rows of a table.", "thead", "tables.html#the-thead-element"),
    ("time", "Associates a date, time, or duration with a machine-readable datetime value.", "time", "text-level-semantics.html#the-time-element"),
    ("title", "Sets the document title used by browser tabs and other interfaces.", "title", "semantics.html#the-title-element"),
    ("tr", "Groups cells into a row of a table.", "tr", "tables.html#the-tr-element"),
    ("track", "Supplies timed text, such as captions, for an audio or video element.", "track", "media.html#the-track-element"),
    ("u", "Marks text with a non-textual annotation, such as a spelling correction.", "u", "text-level-semantics.html#the-u-element"),
    ("ul", "Groups list items whose order is not significant.", "ul", "grouping-content.html#the-ul-element"),
    ("var", "Marks a variable in a mathematical expression or programming context.", "var", "text-level-semantics.html#the-var-element"),
    ("video", "Embeds video with optional playback controls and alternative sources.", "video", "media.html#the-video-element"),
    ("wbr", "Marks an optional line-break opportunity within text.", "wbr", "text-level-semantics.html#the-wbr-element"),
}

/// Resolve only a selected native HTML tag, without allocation.
pub(super) fn lookup(tag: &str) -> Option<&'static ElementDocs> {
    #[cfg(test)]
    LOOKUPS.with(|count| count.set(count.get() + 1));
    // The common DOM tags avoid searching the full descriptor table.
    let index = match tag {
        "a" => 0,
        "button" => 14,
        "div" => 28,
        "img" => 51,
        "input" => 52,
        "p" => 72,
        "section" => 84,
        "span" => 88,
        _ => return lookup_cold(tag),
    };
    Some(&ELEMENTS[index])
}

fn lookup_cold(tag: &str) -> Option<&'static ElementDocs> {
    let mut start = 0;
    let mut end = ELEMENTS.len();
    while start < end {
        let middle = start + (end - start) / 2;
        let entry = &ELEMENTS[middle];
        match entry.tag.cmp(tag) {
            std::cmp::Ordering::Equal => return Some(entry),
            std::cmp::Ordering::Less => start = middle + 1,
            std::cmp::Ordering::Greater => end = middle,
        }
    }
    None
}

#[cfg(test)]
thread_local! {
    static LOOKUPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn take_lookup_count() -> usize {
    LOOKUPS.with(|count| count.replace(0))
}

#[cfg(test)]
mod tests;
