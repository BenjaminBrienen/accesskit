// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

/**
 * The type of an accessibility node.
 *
 * The majority of these roles come from the ARIA specification. Reference
 * the latest draft for proper usage.
 *
 * Like the AccessKit schema as a whole, this list is largely taken
 * from Chromium. However, unlike Chromium's alphabetized list, this list
 * is ordered roughly by expected usage frequency (with the notable exception
 * of [`Role::Unknown`]). This is more efficient in serialization formats
 * where integers use a variable-length encoding.
 */
public enum Role {
    UNKNOWN,
    INLINE_TEXT_BOX,
    CELL,
    STATIC_TEXT,
    IMAGE,
    LINK,
    ROW,
    LIST_ITEM,
    /**
     * Contains the bullet, number, or other marker for a list item.
     */
    LIST_MARKER,
    TREE_ITEM,
    LIST_BOX_OPTION,
    MENU_ITEM,
    MENU_LIST_OPTION,
    PARAGRAPH,
    /**
     * A generic container that should be ignored by assistive technologies
     * and filtered out of platform accessibility trees. Equivalent to the ARIA
     * `none` or `presentation` role, or to an HTML `div` with no role.
     */
    GENERIC_CONTAINER,
    CHECK_BOX,
    RADIO_BUTTON,
    TEXT_INPUT,
    BUTTON,
    DEFAULT_BUTTON,
    PANE,
    ROW_HEADER,
    COLUMN_HEADER,
    COLUMN,
    ROW_GROUP,
    LIST,
    TABLE,
    TABLE_HEADER_CONTAINER,
    LAYOUT_TABLE_CELL,
    LAYOUT_TABLE_ROW,
    LAYOUT_TABLE,
    SWITCH,
    TOGGLE_BUTTON,
    MENU,
    MULTILINE_TEXT_INPUT,
    SEARCH_INPUT,
    DATE_INPUT,
    DATE_TIME_INPUT,
    WEEK_INPUT,
    MONTH_INPUT,
    TIME_INPUT,
    EMAIL_INPUT,
    NUMBER_INPUT,
    PASSWORD_INPUT,
    PHONE_NUMBER_INPUT,
    URL_INPUT,
    ABBR,
    ALERT,
    ALERT_DIALOG,
    APPLICATION,
    ARTICLE,
    AUDIO,
    BANNER,
    BLOCKQUOTE,
    CANVAS,
    CAPTION,
    CARET,
    CODE,
    COLOR_WELL,
    COMBO_BOX,
    EDITABLE_COMBO_BOX,
    COMPLEMENTARY,
    COMMENT,
    CONTENT_DELETION,
    CONTENT_INSERTION,
    CONTENT_INFO,
    DEFINITION,
    DESCRIPTION_LIST,
    DESCRIPTION_LIST_DETAIL,
    DESCRIPTION_LIST_TERM,
    DETAILS,
    DIALOG,
    DIRECTORY,
    DISCLOSURE_TRIANGLE,
    DOCUMENT,
    EMBEDDED_OBJECT,
    EMPHASIS,
    FEED,
    FIGURE_CAPTION,
    FIGURE,
    FOOTER,
    FOOTER_AS_NON_LANDMARK,
    FORM,
    GRID,
    GROUP,
    HEADER,
    HEADER_AS_NON_LANDMARK,
    HEADING,
    IFRAME,
    IFRAME_PRESENTATIONAL,
    IME_CANDIDATE,
    KEYBOARD,
    LEGEND,
    LINE_BREAK,
    LIST_BOX,
    LOG,
    MAIN,
    MARK,
    MARQUEE,
    MATH,
    MENU_BAR,
    MENU_ITEM_CHECK_BOX,
    MENU_ITEM_RADIO,
    MENU_LIST_POPUP,
    METER,
    NAVIGATION,
    NOTE,
    PLUGIN_OBJECT,
    PORTAL,
    PRE,
    PROGRESS_INDICATOR,
    RADIO_GROUP,
    REGION,
    ROOT_WEB_AREA,
    RUBY,
    RUBY_ANNOTATION,
    SCROLL_BAR,
    SCROLL_VIEW,
    SEARCH,
    SECTION,
    SLIDER,
    SPIN_BUTTON,
    SPLITTER,
    STATUS,
    STRONG,
    SUGGESTION,
    SVG_ROOT,
    TAB,
    TAB_LIST,
    TAB_PANEL,
    TERM,
    TIME,
    TIMER,
    TITLE_BAR,
    TOOLBAR,
    TOOLTIP,
    TREE,
    TREE_GRID,
    VIDEO,
    WEB_VIEW,
    WINDOW,
    PDF_ACTIONABLE_HIGHLIGHT,
    PDF_ROOT,
    GRAPHICS_DOCUMENT,
    GRAPHICS_OBJECT,
    GRAPHICS_SYMBOL,
    DOC_ABSTRACT,
    DOC_ACKNOWLEDGEMENTS,
    DOC_AFTERWORD,
    DOC_APPENDIX,
    DOC_BACK_LINK,
    DOC_BIBLIO_ENTRY,
    DOC_BIBLIOGRAPHY,
    DOC_BIBLIO_REF,
    DOC_CHAPTER,
    DOC_COLOPHON,
    DOC_CONCLUSION,
    DOC_COVER,
    DOC_CREDIT,
    DOC_CREDITS,
    DOC_DEDICATION,
    DOC_ENDNOTE,
    DOC_ENDNOTES,
    DOC_EPIGRAPH,
    DOC_EPILOGUE,
    DOC_ERRATA,
    DOC_EXAMPLE,
    DOC_FOOTNOTE,
    DOC_FOREWORD,
    DOC_GLOSSARY,
    DOC_GLOSS_REF,
    DOC_INDEX,
    DOC_INTRODUCTION,
    DOC_NOTE_REF,
    DOC_NOTICE,
    DOC_PAGE_BREAK,
    DOC_PAGE_FOOTER,
    DOC_PAGE_HEADER,
    DOC_PAGE_LIST,
    DOC_PART,
    DOC_PREFACE,
    DOC_PROLOGUE,
    DOC_PULLQUOTE,
    DOC_QNA,
    DOC_SUBTITLE,
    DOC_TIP,
    DOC_TOC,
    /**
     * Behaves similar to an ARIA grid but is primarily used by Chromium's
     * `TableView` and its subclasses, so they can be exposed correctly
     * on certain platforms.
     */
    LIST_GRID,
    /**
     * This is just like a multi-line document, but signals that assistive
     * technologies should implement behavior specific to a VT-100-style
     * terminal.
     */
    TERMINAL
}
