static QUERY: &str = r#"
(
  (namespace_definition
    name: [(namespace_identifier)
    	   (nested_namespace_specifier)] @namespace_name
    body: (declaration_list
      (declaration
        type: (primitive_type) (#EQ? "char")
        declarator: (init_declarator
          declarator: (array_declarator
            declarator: (identifier) @name)
          value: (string_literal) @value
        ))
    )
  )
  (#match? @namespace_name ".*switch.*")
)

(
  (namespace_definition
    name: [(namespace_identifier)
    	   (nested_namespace_specifier)] @namespace_name
    body: (declaration_list
    (preproc_if
    	condition: [(call_expression) (binary_expression)] @cond
        _
      (declaration
        type: (primitive_type) (#EQ? "char")
        declarator: (init_declarator
          declarator: (array_declarator
            declarator: (identifier) @name)
          value: (_) @value
        ))
      )
    )
  )
  (#match? @namespace_name ".*switch.*")
)
"#;

use std::{env::args, error::Error};

use hyper_ast::{
    self,
    position::TreePath,
    store::defaults::NodeIdentifier,
    types::{HyperAST, HyperType, LabelStore, NodeStore},
};
use hyper_ast_cvs_git::{cpp_processor::SUB_QUERIES, multi_preprocessed::PreProcessedRepositories};
use hyper_ast_gen_ts_cpp::{
    legion::CppTreeGen,
    types::{as_any, Type},
};
use hyper_ast_tsquery::Language;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // // SAFETY: trivial when first thing in main
    // // only process the *switches named files
    // unsafe { hyper_ast_cvs_git::processing::file_sys::ONLY_SWITCHES = true };
    println!("{}", QUERY);

    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // .with_env_filter("hyper_ast_cvs_git=trace,chromium-switches=trace")
        .try_init()
        .unwrap();

    let args: Vec<_> = args().skip(1).collect();
    dbg!(&args);

    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo("chromium", "chromium");
    let config = hyper_ast_cvs_git::processing::RepoConfig::CppMake;
    const DEFAULT_OID: &str = "f461f9752e5918c5c87f2e3767bcb24945ee0fa0";
    let commit = args.get(0).map_or(DEFAULT_OID, |x| x.as_ref());
    let query = QUERY;
    let language = "Cpp";
    let mut repositories = PreProcessedRepositories::default();
    let language = hyper_ast_cvs_git::resolve_language(&language).unwrap();
    let code = ast_from_repo(&mut repositories, repo_spec, config, commit)?;
    // let code = ast_from_text(&mut repositories, EXAMPLE_SPACING, language.clone());

    println!("parsing_time: {}", repositories.processor.parsing_time.as_secs_f64());
    println!("processing_time: {}", repositories.processor.processing_time.as_secs_f64());

    querying_cpp(&repositories, code, language, query, true)
}

#[test]
fn test_spacing_issue() -> Result<(), Box<dyn std::error::Error>> {
    use hyper_ast_cvs_git::preprocessed::{child_by_name, child_by_type};
    // let _ = tracing_subscriber::fmt()
    //     .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //     // .with_env_filter("hyper_ast_cvs_git=trace,chromium-switches=trace")
    //     .try_init()
    //     .unwrap();
    let query = r#"(preproc_if) @root"#;
    let language = "Cpp";
    let mut repositories = PreProcessedRepositories::default();
    let language = hyper_ast_cvs_git::resolve_language(&language).unwrap();
    let code = ast_from_text(&mut repositories, EXAMPLE_SPACING, language.clone());
    let stores = &repositories.processor.main_stores;
    // let code = child_by_name(stores, code, "url").unwrap();
    // let code = child_by_name(stores, code, "gurl.h").unwrap();
    println!(
        "  @root     {}",
        hyper_ast::nodes::SimpleSerializer::<_, _, true, true, false, false, true>::new(
            stores, code
        )
    );
    // let code = child_by_type(stores, code, &as_any(&Type::NamespaceDefinition))
    //     .unwrap()
    //     .0;
    compare_querying_with_and_without_skipping(&repositories, code, language, query)
}

#[ignore]
#[test]
fn test_query_incr_whole_commit() -> Result<(), Box<dyn std::error::Error>> {
    // SAFETY: trivial when first thing in test
    // only process the *switches named files
    unsafe { hyper_ast_cvs_git::processing::file_sys::ONLY_SWITCHES = true };
    // let _ = tracing_subscriber::fmt()
    //     .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //     // .with_env_filter("hyper_ast_cvs_git=trace,chromium-switches=trace")
    //     .try_init()
    //     .unwrap();
    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo("chromium", "chromium");
    let config = hyper_ast_cvs_git::processing::RepoConfig::CppMake;
    let commit = "f461f9752e5918c5c87f2e3767bcb24945ee0fa0";
    let query = r#"(preproc_if) @root"#;
    let language = "Cpp";
    let mut repositories = PreProcessedRepositories::default();
    let language = hyper_ast_cvs_git::resolve_language(&language).unwrap();
    let code = ast_from_repo(&mut repositories, repo_spec, config, commit)?;
    compare_querying_with_and_without_skipping(&repositories, code, language, query)
}
#[test]
fn test_query_incr_exact() -> Result<(), Box<dyn std::error::Error>> {
    use hyper_ast_cvs_git::preprocessed::{child_by_name, child_by_type};
    // let _ = tracing_subscriber::fmt()
    //     .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //     // .with_env_filter("hyper_ast_cvs_git=trace,chromium-switches=trace")
    //     .try_init()
    //     .unwrap();

    let query = r#"(preproc_if) @root"#;
    let language = "Cpp";
    let mut repositories = PreProcessedRepositories::default();
    let language = hyper_ast_cvs_git::resolve_language(&language).unwrap();
    let code = ast_from_text(&mut repositories, EXAMPLE1, language.clone());
    let stores = &repositories.processor.main_stores;
    // let code = child_by_type(stores, code, &as_any(&Type::NamespaceDefinition))
    //     .unwrap()
    //     .0;
    compare_querying_with_and_without_skipping(&repositories, code, language, query)
}
#[test]
fn test_query_if_statement() -> Result<(), Box<dyn std::error::Error>> {
    use hyper_ast_cvs_git::preprocessed::{child_by_name, child_by_type};
    // let _ = tracing_subscriber::fmt()
    //     .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //     // .with_env_filter("hyper_ast_cvs_git=trace,chromium-switches=trace")
    //     .try_init()
    //     .unwrap();

    let query = r#"(preproc_if
  condition: (identifier) @name  
  ) @root"#;
    let language = "Cpp";
    let mut repositories = PreProcessedRepositories::default();
    let language = hyper_ast_cvs_git::resolve_language(&language).unwrap();
    let code = ast_from_text(&mut repositories, EXAMPLE1, language.clone());
    let stores = &repositories.processor.main_stores;
    // let code = child_by_type(stores, code, &as_any(&Type::NamespaceDefinition))
    //     .unwrap()
    //     .0;
    querying_cpp(&repositories, code, language, query, false)
}

/// use this in a test if you suspect a querying discrepancy on a commit due to the subtree skipping feature,
/// it might help you find where the query verdicts where not bubbled up.
fn compare_querying_with_and_without_skipping(
    repositories: &PreProcessedRepositories,
    code: NodeIdentifier,
    language: Language,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let query_incr =
        hyper_ast_tsquery::Query::with_precomputed(&query, language.clone(), SUB_QUERIES)
            .map(|x| x.1)
            .unwrap();
    let query = hyper_ast_tsquery::Query::new(&query, language.clone()).unwrap();
    let stores = &repositories.processor.main_stores;
    eprintln!(
        "  @root     {}",
        hyper_ast::nodes::SimpleSerializer::<_, _, true, true, false, false, true>::new(
            stores, code
        )
    );
    eprintln!(
        "  @root     {}",
        hyper_ast::nodes::TextSerializer::new(stores, code)
    );
    let mut qcursor_incr = {
        let pos = hyper_ast::position::StructuralPosition::new(code);
        let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
        query_incr.matches(cursor)
    }
    .into_iter();
    let mut qcursor = {
        let pos = hyper_ast::position::StructuralPosition::new(code);
        let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
        query.matches(cursor)
    }
    .into_iter();
    let root_cid = query_incr.capture_index_for_name("root").unwrap();
    let mut count = 0;
    loop {
        let m = qcursor.next();
        if m.is_none() {
            eprintln!("match count: {count}");
            return Ok(());
        }
        count += 1;
        let m = &m
            .as_ref()
            .unwrap()
            .nodes_for_capture_index(root_cid)
            .next()
            .unwrap()
            .pos;
        // log::debug!("m: {:?}", m.make_file_line_range(stores));
        eprintln!(
            "  @root     {}",
            hyper_ast::nodes::SimpleSerializer::<_, _, true, true, false, false, true>::new(
                stores,
                *m.node().unwrap()
            )
        );
        eprintln!(
            "  @root     {}",
            hyper_ast::nodes::TextSerializer::new(stores, *m.node().unwrap())
        );

        let m_incr = qcursor_incr.next();
        let m_incr = &m_incr
            .as_ref()
            .unwrap()
            .nodes_for_capture_index(root_cid)
            .next()
            .unwrap()
            .pos;
        log::debug!("m_incr: {:?}", m_incr);
        log::debug!("m_incr: {:?}", m_incr.make_file_line_range(stores));
    }
}

fn querying_cpp(
    repositories: &PreProcessedRepositories,
    code: NodeIdentifier,
    language: Language,
    query: &str,
    incr: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let stores = &repositories.processor.main_stores;
    let query = if incr {
        hyper_ast_tsquery::Query::with_precomputed(&query, language.clone(), SUB_QUERIES)
            .map(|x| x.1)
            .unwrap()
    } else {
        hyper_ast_tsquery::Query::new(&query, language.clone()).unwrap()
    };
    let mut qcursor = {
        let pos = hyper_ast::position::StructuralPosition::new(code);
        let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
        query.matches(cursor)
    }
    .into_iter();
    let name_cid = query.capture_index_for_name("name").unwrap();
    let value_cid = query.capture_index_for_name("value").unwrap();
    let cond_cid = query.capture_index_for_name("cond").unwrap();
    let namespace_name_cid = query.capture_index_for_name("namespace_name").unwrap();
    let mut count = 0;
    loop {
        let m = qcursor.next();
        if m.is_none() {
            eprintln!("match count: {count}");
            return Ok(());
        }
        count += 1;
        let name = &m
            .as_ref()
            .unwrap()
            .nodes_for_capture_index(name_cid)
            .next()
            .unwrap()
            .pos;
        let (file, startl, _) = name.make_file_line_range(stores);
        eprintln!("{}:{}", file, startl);
        eprintln!(
            "  @name     {}",
            hyper_ast::nodes::SimpleSerializer::<_, _, true, true, false, false, true>::new(
                stores,
                *name.node().unwrap()
            )
        );
        eprintln!(
            "  @name     {}",
            hyper_ast::nodes::TextSerializer::new(stores, *name.node().unwrap())
        );
        let value = &m
            .as_ref()
            .unwrap()
            .nodes_for_capture_index(value_cid)
            .next()
            .unwrap()
            .pos;
        eprintln!(
            "  @value    {}",
            hyper_ast::nodes::SimpleSerializer::<_, _, true, true, false, false, true>::new(
                stores,
                *value.node().unwrap()
            )
        );
        eprintln!(
            "  @value    {}",
            hyper_ast::nodes::TextSerializer::new(stores, *value.node().unwrap())
        );
        let parent = value.parent().unwrap();
        let id = value.node().unwrap();
        let ty = stores.resolve_type(id);
        dbg!(ty);
        dbg!(ty.is_hidden());
        dbg!(ty.is_named());
        dbg!(ty.is_supertype());

        let m_cond = m.as_ref().unwrap().nodes_for_capture_index(cond_cid).next();
        if let Some(cond) = m_cond {
            let cond = &m_cond.unwrap().pos;
            eprintln!(
                "  @cond     {}",
                hyper_ast::nodes::SimpleSerializer::<_, _, true, true, false, false, true>::new(
                    stores,
                    *cond.node().unwrap()
                )
            );
            eprintln!(
                "  @cond     {}",
                hyper_ast::nodes::TextSerializer::new(stores, *cond.node().unwrap())
            );
        }
        let namespace_name = &m
            .as_ref()
            .unwrap()
            .nodes_for_capture_index(namespace_name_cid)
            .next()
            .unwrap()
            .pos;
        eprintln!(
            "  @namespace_name     {}",
            hyper_ast::nodes::SimpleSerializer::<_, _, true, true, false, false, true>::new(
                stores,
                *namespace_name.node().unwrap()
            )
        );
        eprintln!(
            "  @namespace_name     {}",
            hyper_ast::nodes::TextSerializer::new(stores, *namespace_name.node().unwrap())
        );
    }
}

fn ast_from_text(
    repositories: &mut PreProcessedRepositories,
    text: &str,
    language: hyper_ast_tsquery::Language,
) -> NodeIdentifier {
    let stores = &mut repositories
        .processor
        .main_stores
        .mut_with_ts::<hyper_ast_gen_ts_cpp::types::TStore>();
    let mut md_cache = Default::default();
    let precomputeds = SUB_QUERIES;
    static DQ: &str = "(_)";
    let (precomp, _) =
        hyper_ast_tsquery::Query::with_precomputed(DQ, language, precomputeds).unwrap();
    let mut tree_gen = CppTreeGen::new(stores, &mut md_cache).with_more(precomp);
    let tree = hyper_ast_gen_ts_cpp::legion::tree_sitter_parse(text.as_bytes());
    let tree = match tree {
        Ok(tree) => {
            println!("{}", tree.root_node().to_sexp());
            tree
        }
        Err(tree) => {
            println!("{}", tree.root_node().to_sexp());
            tree
        }
    };
    tree_gen
        .generate_file(b"", text.as_bytes(), tree.walk())
        .local
        .compressed_node
}

fn ast_from_repo(
    repositories: &mut PreProcessedRepositories,
    repo_spec: hyper_ast_cvs_git::git::Repo,
    config: hyper_ast_cvs_git::processing::RepoConfig,
    commit: &str,
) -> Result<NodeIdentifier, Box<dyn Error>> {
    repositories.register_config(repo_spec.clone(), config);
    let repo = repositories
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repository = repo.nofetch();
    let commits = repositories.pre_process_with_limit(&mut repository, "", &commit, 1)?;
    let commit = repositories
        .get_commit(&repository.config, &commits[0])
        .unwrap();
    let code = commit.ast_root;
    Ok(code)
}

// sorten version of ui/base/ui_base_switches.cc:0
static EXAMPLE0: &str = r#"
namespace switches {

#if BUILDFLAG(IS_ANDROID)
#endif

}  // namespace switches
"#;

// ui/base/ui_base_switches.cc:0
static EXAMPLE1: &str = r#"
namespace switches {

#if BUILDFLAG(IS_ANDROID)
// Disable overscroll edge effects like those found in Android views.
const char kDisableOverscrollEdgeEffect[] = "disable-overscroll-edge-effect";

// Disable the pull-to-refresh effect when vertically overscrolling content.
const char kDisablePullToRefreshEffect[] = "disable-pull-to-refresh-effect";
#endif

#if BUILDFLAG(IS_MAC)
// Disable animations for showing and hiding modal dialogs.
const char kDisableModalAnimations[] = "disable-modal-animations";

// Show borders around CALayers corresponding to overlays and partial damage.
const char kShowMacOverlayBorders[] = "show-mac-overlay-borders";
#endif

#if BUILDFLAG(IS_LINUX) || BUILDFLAG(IS_CHROMEOS)
// Specifies system font family name. Improves determenism when rendering
// pages in headless mode.
const char kSystemFontFamily[] = "system-font-family";
#endif

#if BUILDFLAG(IS_LINUX)
// Specify the toolkit used to construct the Linux GUI.
const char kUiToolkitFlag[] = "ui-toolkit";
// Disables GTK IME integration.
const char kDisableGtkIme[] = "disable-gtk-ime";
#endif

// Disables touch event based drag and drop.
const char kDisableTouchDragDrop[] = "disable-touch-drag-drop";

// Disable re-use of non-exact resources to fulfill ResourcePool requests.
// Intended only for use in layout or pixel tests to reduce noise.
const char kDisallowNonExactResourceReuse[] =
    "disallow-non-exact-resource-reuse";

// Treats DRM virtual connector as external to enable display mode change in VM.
const char kDRMVirtualConnectorIsExternal[] =
    "drm-virtual-connector-is-external";

// Enables touch event based drag and drop.
const char kEnableTouchDragDrop[] = "enable-touch-drag-drop";

// Forces the caption style for WebVTT captions.
const char kForceCaptionStyle[] = "force-caption-style";

// Forces dark mode in UI for platforms that support it.
const char kForceDarkMode[] = "force-dark-mode";

// Forces high-contrast mode in native UI drawing, regardless of system
// settings. Note that this has limited effect on Windows: only Aura colors will
// be switched to high contrast, not other system colors.
const char kForceHighContrast[] = "force-high-contrast";

// The language file that we want to try to open. Of the form
// language[-country] where language is the 2 letter code from ISO-639.
// On Linux, this flag does not work; use the LC_*/LANG environment variables
// instead.
const char kLang[] = "lang";

// Transform localized strings to be longer, with beginning and end markers to
// make truncation visually apparent.
const char kMangleLocalizedStrings[] = "mangle-localized-strings";

// Visualize overdraw by color-coding elements based on if they have other
// elements drawn underneath. This is good for showing where the UI might be
// doing more rendering work than necessary. The colors are hinting at the
// amount of overdraw on your screen for each pixel, as follows:
//
// True color: No overdraw.
// Blue: Overdrawn once.
// Green: Overdrawn twice.
// Pink: Overdrawn three times.
// Red: Overdrawn four or more times.
const char kShowOverdrawFeedback[] = "show-overdraw-feedback";

// Re-draw everything multiple times to simulate a much slower machine.
// Give a slow down factor to cause renderer to take that many times longer to
// complete, such as --slow-down-compositing-scale-factor=2.
const char kSlowDownCompositingScaleFactor[] =
    "slow-down-compositing-scale-factor";

// Tint composited color.
const char kTintCompositedContent[] = "tint-composited-content";

// Controls touch-optimized UI layout for top chrome.
const char kTopChromeTouchUi[] = "top-chrome-touch-ui";
const char kTopChromeTouchUiAuto[] = "auto";
const char kTopChromeTouchUiDisabled[] = "disabled";
const char kTopChromeTouchUiEnabled[] = "enabled";

// Disable partial swap which is needed for some OpenGL drivers / emulators.
const char kUIDisablePartialSwap[] = "ui-disable-partial-swap";

// Enables the ozone x11 clipboard for linux-chromeos.
const char kUseSystemClipboard[] = "use-system-clipboard";

}  // namespace switches
"#;

// derived from EXAMPLE_SPACING_SAVE
// mostly issue with the class_decl
// where the preproc call makes tree sitter think it is a function definition
static EXAMPLE_SPACING: &str = r#"
class G {};
bool operator==(const G& x, t spec);
namespace d {
class A() B {
  B& operator=(const B&) = delete;
};
}  // namespace d
"#;

// url/gurl.h
static EXAMPLE_SPACING_SAVE: &str = r#"// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef URL_GURL_H_
#define URL_GURL_H_

#include <stddef.h>

#include <iosfwd>
#include <memory>
#include <string>
#include <string_view>

#include "base/component_export.h"
#include "base/debug/alias.h"
#include "base/debug/crash_logging.h"
#include "base/trace_event/base_tracing_forward.h"
#include "url/third_party/mozilla/url_parse.h"
#include "url/url_canon.h"
#include "url/url_canon_stdstring.h"
#include "url/url_constants.h"

// Represents a URL. GURL is Google's URL parsing library.
//
// A parsed canonicalized URL is guaranteed to be UTF-8. Any non-ASCII input
// characters are UTF-8 encoded and % escaped to ASCII.
//
// The string representation of a URL is called the spec(). Getting the
// spec will assert if the URL is invalid to help protect against malicious
// URLs. If you want the "best effort" canonicalization of an invalid URL, you
// can use possibly_invalid_spec(). Test validity with is_valid(). Data and
// javascript URLs use GetContent() to extract the data.
//
// This class has existence checkers and getters for the various components of
// a URL. Existence is different than being nonempty. "http://www.google.com/?"
// has a query that just happens to be empty, and has_query() will return true
// while the query getters will return the empty string.
//
// Prefer not to modify a URL using string operations (though sometimes this is
// unavoidable). Instead, use ReplaceComponents which can replace or delete
// multiple parts of a URL in one step, doesn't re-canonicalize unchanged
// sections, and avoids some screw-ups. An example is creating a URL with a
// path that contains a literal '#'. Using string concatenation will generate a
// URL with a truncated path and a reference fragment, while ReplaceComponents
// will know to escape this and produce the desired result.
//
// WARNING: While there is no length limit on GURLs, the Mojo serialization
// code will replace any very long URL with an invalid GURL.
// See url::mojom::kMaxURLChars for more details.
class COMPONENT_EXPORT(URL) GURL {
 public:
  using Replacements = url::StringViewReplacements<char>;
  using ReplacementsW = url::StringViewReplacements<char16_t>;

  // Creates an empty, invalid URL.
  GURL();

  // Copy construction is relatively inexpensive, with most of the time going
  // to reallocating the string. It does not re-parse.
  GURL(const GURL& other);
  GURL(GURL&& other) noexcept;

  // The strings to this constructor should be UTF-8 / UTF-16. They will be
  // parsed and canonicalized. For example, the host is lower cased, and
  // characters may be percent-encoded or percent-decoded to normalize the URL.
  explicit GURL(std::string_view url_string);
  explicit GURL(std::u16string_view url_string);

  // Constructor for URLs that have already been parsed and canonicalized. This
  // is used for conversions from KURL, for example. The caller must supply all
  // information associated with the URL, which must be correct and consistent.
  GURL(const char* canonical_spec,
       size_t canonical_spec_len,
       const url::Parsed& parsed,
       bool is_valid);
  // Notice that we take the canonical_spec by value so that we can convert
  // from WebURL without copying the string. When we call this constructor
  // we pass in a temporary std::string, which lets the compiler skip the
  // copy and just move the std::string into the function argument. In the
  // implementation, we use std::move to move the data into the GURL itself,
  // which means we end up with zero copies.
  GURL(std::string canonical_spec, const url::Parsed& parsed, bool is_valid);

  ~GURL();

  GURL& operator=(const GURL& other);
  GURL& operator=(GURL&& other) noexcept;

  // Returns true when this object represents a valid parsed URL. When not
  // valid, other functions will still succeed, but you will not get canonical
  // data out in the format you may be expecting. Instead, we keep something
  // "reasonable looking" so that the user can see how it's busted if
  // displayed to them.
  bool is_valid() const {
    return is_valid_;
  }

  // Returns true if the URL is zero-length. Note that empty URLs are also
  // invalid, and is_valid() will return false for them. This is provided
  // because some users may want to treat the empty case differently.
  bool is_empty() const {
    return spec_.empty();
  }

  // Returns the raw spec, i.e., the full text of the URL, in canonical UTF-8,
  // if the URL is valid. If the URL is not valid, this will assert and return
  // the empty string (for safety in release builds, to keep them from being
  // misused which might be a security problem).
  //
  // The URL will be ASCII (non-ASCII characters will be %-escaped UTF-8).
  //
  // The exception is for empty() URLs (which are !is_valid()) but this will
  // return the empty string without asserting.
  //
  // Use invalid_spec() below to get the unusable spec of an invalid URL. This
  // separation is designed to prevent errors that may cause security problems
  // that could result from the mistaken use of an invalid URL.
  const std::string& spec() const;

  // Returns the potentially invalid spec for a the URL. This spec MUST NOT be
  // modified or sent over the network. It is designed to be displayed in error
  // messages to the user, as the appearance of the spec may explain the error.
  // If the spec is valid, the valid spec will be returned.
  //
  // The returned string is guaranteed to be valid UTF-8.
  const std::string& possibly_invalid_spec() const {
    return spec_;
  }

  // Getter for the raw parsed structure. This allows callers to locate parts
  // of the URL within the spec themselves. Most callers should consider using
  // the individual component getters below.
  //
  // The returned parsed structure will reference into the raw spec, which may
  // or may not be valid. If you are using this to index into the spec, BE
  // SURE YOU ARE USING possibly_invalid_spec() to get the spec, and that you
  // don't do anything "important" with invalid specs.
  const url::Parsed& parsed_for_possibly_invalid_spec() const {
    return parsed_;
  }

  // Allows GURL to used as a key in STL (for example, a std::set or std::map).
  constexpr friend auto operator<=>(const GURL& lhs, const GURL& rhs) {
    return lhs.spec_ <=> rhs.spec_;
  }

  // Resolves a URL that's possibly relative to this object's URL, and returns
  // it. Absolute URLs are also handled according to the rules of URLs on web
  // pages.
  //
  // It may be impossible to resolve the URLs properly. If the input is not
  // "standard" (IsStandard() == false) and the input looks relative, we can't
  // resolve it. In these cases, the result will be an empty, invalid GURL.
  //
  // The result may also be a nonempty, invalid URL if the input has some kind
  // of encoding error. In these cases, we will try to construct a "good" URL
  // that may have meaning to the user, but it will be marked invalid.
  //
  // It is an error to resolve a URL relative to an invalid URL. The result
  // will be the empty URL.
  [[nodiscard]] GURL Resolve(std::string_view relative) const;
  [[nodiscard]] GURL Resolve(std::u16string_view relative) const;

  // Creates a new GURL by replacing the current URL's components with the
  // supplied versions. See the Replacements class in url_canon.h for more.
  //
  // These are not particularly quick, so avoid doing mutations when possible.
  // Prefer the 8-bit version when possible.
  //
  // It is an error to replace components of an invalid URL. The result will
  // be the empty URL.
  //
  // Note that this intentionally disallows direct use of url::Replacements,
  // which is harder to use correctly.
  [[nodiscard]] GURL ReplaceComponents(const Replacements& replacements) const;
  [[nodiscard]] GURL ReplaceComponents(const ReplacementsW& replacements) const;

  // A helper function that is equivalent to replacing the path with a slash
  // and clearing out everything after that. We sometimes need to know just the
  // scheme and the authority. If this URL is not a standard URL (it doesn't
  // have the regular authority and path sections), then the result will be
  // an empty, invalid GURL. Note that this *does* work for file: URLs, which
  // some callers may want to filter out before calling this.
  //
  // It is an error to get an empty path on an invalid URL. The result
  // will be the empty URL.
  [[nodiscard]] GURL GetWithEmptyPath() const;

  // A helper function to return a GURL without the filename, query values, and
  // fragment. For example,
  // GURL("https://www.foo.com/index.html?q=test").GetWithoutFilename().spec()
  // will return "https://www.foo.com/".
  // GURL("https://www.foo.com/bar/").GetWithoutFilename().spec()
  // will return "https://www.foo.com/bar/". If the GURL is invalid or missing a
  // scheme, authority or path, it will return an empty, invalid GURL.
  [[nodiscard]] GURL GetWithoutFilename() const;

  // A helper function to return a GURL without the Ref (also named Fragment
  // Identifier). For example,
  // GURL("https://www.foo.com/index.html#test").GetWithoutRef().spec()
  // will return "https://www.foo.com/index.html".
  // If the GURL is invalid or missing a
  // scheme, authority or path, it will return an empty, invalid GURL.
  [[nodiscard]] GURL GetWithoutRef() const;

  // A helper function to return a GURL containing just the scheme, host,
  // and port from a URL. Equivalent to clearing any username and password,
  // replacing the path with a slash, and clearing everything after that. If
  // this URL is not a standard URL, then the result will be an empty,
  // invalid GURL. If the URL has neither username nor password, this
  // degenerates to GetWithEmptyPath().
  //
  // It is an error to get the origin of an invalid URL. The result
  // will be the empty URL.
  //
  // WARNING: Please avoid converting urls into origins if at all possible!
  // //docs/security/origin-vs-url.md is a list of gotchas that can result. Such
  // conversions will likely return a wrong result for about:blank and/or
  // in the presence of iframe.sandbox attribute. Prefer to get origins directly
  // from the source (e.g. RenderFrameHost::GetLastCommittedOrigin).
  [[nodiscard]] GURL DeprecatedGetOriginAsURL() const;

  // A helper function to return a GURL stripped from the elements that are not
  // supposed to be sent as HTTP referrer: username, password and ref fragment.
  // For invalid URLs or URLs that no valid referrers, an empty URL will be
  // returned.
  [[nodiscard]] GURL GetAsReferrer() const;

  // Returns true if the scheme for the current URL is a known "standard-format"
  // scheme. A standard-format scheme adheres to what RFC 3986 calls "generic
  // URI syntax" (https://tools.ietf.org/html/rfc3986#section-3). This includes
  // file: and filesystem:, which some callers may want to filter out explicitly
  // by calling SchemeIsFile[System].
  bool IsStandard() const;

  // Returns true when the url is of the form about:blank, about:blank?foo or
  // about:blank/#foo.
  bool IsAboutBlank() const;

  // Returns true when the url is of the form about:srcdoc, about:srcdoc?foo or
  // about:srcdoc/#foo.
  bool IsAboutSrcdoc() const;

  // Returns true if the given parameter (should be lower-case ASCII to match
  // the canonicalized scheme) is the scheme for this URL. Do not include a
  // colon.
  bool SchemeIs(std::string_view lower_ascii_scheme) const;

  // Returns true if the scheme is "http" or "https".
  bool SchemeIsHTTPOrHTTPS() const;

  // Returns true is the scheme is "ws" or "wss".
  bool SchemeIsWSOrWSS() const;

  // We often need to know if this is a file URL. File URLs are "standard", but
  // are often treated separately by some programs.
  bool SchemeIsFile() const {
    return SchemeIs(url::kFileScheme);
  }

  // FileSystem URLs need to be treated differently in some cases.
  bool SchemeIsFileSystem() const {
    return SchemeIs(url::kFileSystemScheme);
  }

  // Returns true if the scheme indicates a network connection that uses TLS or
  // some other cryptographic protocol (e.g. QUIC) for security.
  //
  // This function is a not a complete test of whether or not an origin's code
  // is minimally trustworthy. For that, see Chromium's |IsOriginSecure| for a
  // higher-level and more complete semantics. See that function's documentation
  // for more detail.
  bool SchemeIsCryptographic() const;

  // As above, but static. Parameter should be lower-case ASCII.
  static bool SchemeIsCryptographic(std::string_view lower_ascii_scheme);

  // Returns true if the scheme is "blob".
  bool SchemeIsBlob() const {
    return SchemeIs(url::kBlobScheme);
  }

  // Returns true if the scheme is a local scheme, as defined in Fetch:
  // https://fetch.spec.whatwg.org/#local-scheme
  bool SchemeIsLocal() const;

  // For most URLs, the "content" is everything after the scheme (skipping the
  // scheme delimiting colon) and before the fragment (skipping the fragment
  // delimiting octothorpe). For javascript URLs the "content" also includes the
  // fragment delimiter and fragment.
  //
  // It is an error to get the content of an invalid URL: the result will be an
  // empty string.
  //
  // Important note: The feature flag,
  // url::kStandardCompliantNonSpecialSchemeURLParsing, changes the behavior of
  // GetContent() and GetContentPiece() for some non-special URLs. See
  // GURLTest::ContentForNonStandardURLs for the differences.
  //
  // Until the flag becomes enabled by default, you'll need to manually check
  // the flag when using GetContent() and GetContentPiece() for non-special
  // URLs. See http://crbug.com/40063064 for more details.
  std::string GetContent() const;
  std::string_view GetContentPiece() const;

  // Returns true if the hostname is an IP address. Note: this function isn't
  // as cheap as a simple getter because it re-parses the hostname to verify.
  bool HostIsIPAddress() const;

  // Not including the colon. If you are comparing schemes, prefer SchemeIs.
  bool has_scheme() const { return parsed_.scheme.is_valid(); }
  std::string scheme() const {
    return ComponentString(parsed_.scheme);
  }
  std::string_view scheme_piece() const {
    return ComponentStringPiece(parsed_.scheme);
  }

  bool has_username() const { return parsed_.username.is_valid(); }
  std::string username() const {
    return ComponentString(parsed_.username);
  }
  std::string_view username_piece() const {
    return ComponentStringPiece(parsed_.username);
  }

  bool has_password() const { return parsed_.password.is_valid(); }
  std::string password() const {
    return ComponentString(parsed_.password);
  }
  std::string_view password_piece() const {
    return ComponentStringPiece(parsed_.password);
  }

  // The host may be a hostname, an IPv4 address, or an IPv6 literal surrounded
  // by square brackets, like "[2001:db8::1]". To exclude these brackets, use
  // HostNoBrackets() below.
  bool has_host() const {
    // Note that hosts are special, absence of host means length 0.
    return parsed_.host.is_nonempty();
  }
  std::string host() const {
    return ComponentString(parsed_.host);
  }
  std::string_view host_piece() const {
    return ComponentStringPiece(parsed_.host);
  }

  // The port if one is explicitly specified. Most callers will want IntPort()
  // or EffectiveIntPort() instead of these. The getters will not include the
  // ':'.
  bool has_port() const { return parsed_.port.is_valid(); }
  std::string port() const {
    return ComponentString(parsed_.port);
  }
  std::string_view port_piece() const {
    return ComponentStringPiece(parsed_.port);
  }

  // Including first slash following host, up to the query. The URL
  // "http://www.google.com/" has a path of "/".
  bool has_path() const { return parsed_.path.is_valid(); }
  std::string path() const {
    return ComponentString(parsed_.path);
  }
  std::string_view path_piece() const {
    return ComponentStringPiece(parsed_.path);
  }

  // Stuff following '?' up to the ref. The getters will not include the '?'.
  bool has_query() const { return parsed_.query.is_valid(); }
  std::string query() const {
    return ComponentString(parsed_.query);
  }
  std::string_view query_piece() const {
    return ComponentStringPiece(parsed_.query);
  }

  // Stuff following '#' to the end of the string. This will be %-escaped UTF-8.
  // The getters will not include the '#'.
  bool has_ref() const { return parsed_.ref.is_valid(); }
  std::string ref() const {
    return ComponentString(parsed_.ref);
  }
  std::string_view ref_piece() const {
    return ComponentStringPiece(parsed_.ref);
  }

  // Returns a parsed version of the port. Can also be any of the special
  // values defined in Parsed for ExtractPort.
  int IntPort() const;

  // Returns the port number of the URL, or the default port number.
  // If the scheme has no concept of port (or unknown default) returns
  // PORT_UNSPECIFIED.
  int EffectiveIntPort() const;

  // Extracts the filename portion of the path and returns it. The filename
  // is everything after the last slash in the path. This may be empty.
  std::string ExtractFileName() const;

  // Returns the path that should be sent to the server. This is the path,
  // parameter, and query portions of the URL. It is guaranteed to be ASCII.
  std::string PathForRequest() const;

  // Returns the same characters as PathForRequest(), avoiding a copy.
  std::string_view PathForRequestPiece() const;

  // Returns the host, excluding the square brackets surrounding IPv6 address
  // literals. This can be useful for passing to getaddrinfo().
  std::string HostNoBrackets() const;

  // Returns the same characters as HostNoBrackets(), avoiding a copy.
  std::string_view HostNoBracketsPiece() const;

  // Returns true if this URL's host matches or is in the same domain as
  // the given input string. For example, if the hostname of the URL is
  // "www.google.com", this will return true for "com", "google.com", and
  // "www.google.com".
  //
  // The input domain should match host canonicalization rules. i.e. the input
  // should be lowercase except for escape chars.
  //
  // This call is more efficient than getting the host and checking whether the
  // host has the specific domain or not because no copies or object
  // constructions are done.
  bool DomainIs(std::string_view canonical_domain) const;

  // Checks whether or not two URLs differ only in the ref (the part after
  // the # character).
  bool EqualsIgnoringRef(const GURL& other) const;

  // Swaps the contents of this GURL object with |other|, without doing
  // any memory allocations.
  void Swap(GURL* other);

  // Returns a reference to a singleton empty GURL. This object is for callers
  // who return references but don't have anything to return in some cases.
  // If you just want an empty URL for normal use, prefer GURL(). This function
  // may be called from any thread.
  static const GURL& EmptyGURL();

  // Returns the inner URL of a nested URL (currently only non-null for
  // filesystem URLs).
  //
  // TODO(mmenke): inner_url().spec() currently returns the same value as
  // caling spec() on the GURL itself. This should be fixed.
  // See https://crbug.com/619596
  const GURL* inner_url() const {
    return inner_url_.get();
  }

  // Estimates dynamic memory usage.
  // See base/trace_event/memory_usage_estimator.h for more info.
  size_t EstimateMemoryUsage() const;

  // Helper used by GURL::IsAboutUrl and KURL::IsAboutURL.
  static bool IsAboutPath(std::string_view actual_path,
                          std::string_view allowed_path);

  void WriteIntoTrace(perfetto::TracedValue context) const;

 private:
  // Variant of the string parsing constructor that allows the caller to elect
  // retain trailing whitespace, if any, on the passed URL spec, but only if
  // the scheme is one that allows trailing whitespace. The primary use-case is
  // for data: URLs. In most cases, you want to use the single parameter
  // constructor above.
  enum RetainWhiteSpaceSelector { RETAIN_TRAILING_PATH_WHITEPACE };
  GURL(const std::string& url_string, RetainWhiteSpaceSelector);

  template <typename T, typename CharT = typename T::value_type>
  void InitCanonical(T input_spec, bool trim_path_end);

  void InitializeFromCanonicalSpec();

  // Helper used by IsAboutBlank and IsAboutSrcdoc.
  bool IsAboutUrl(std::string_view allowed_path) const;

  // Returns the substring of the input identified by the given component.
  std::string ComponentString(const url::Component& comp) const {
    return std::string(ComponentStringPiece(comp));
  }
  std::string_view ComponentStringPiece(const url::Component& comp) const {
    if (comp.is_empty())
      return std::string_view();
    return std::string_view(spec_).substr(static_cast<size_t>(comp.begin),
                                          static_cast<size_t>(comp.len));
  }

  void ProcessFileSystemURLAfterReplaceComponents();

  // The actual text of the URL, in canonical ASCII form.
  std::string spec_;

  // Set when the given URL is valid. Otherwise, we may still have a spec and
  // components, but they may not identify valid resources (for example, an
  // invalid port number, invalid characters in the scheme, etc.).
  bool is_valid_;

  // Identified components of the canonical spec.
  url::Parsed parsed_;

  // Used for nested schemes [currently only filesystem:].
  std::unique_ptr<GURL> inner_url_;
};

// Stream operator so GURL can be used in assertion statements.
COMPONENT_EXPORT(URL)
std::ostream& operator<<(std::ostream& out, const GURL& url);

COMPONENT_EXPORT(URL) bool operator==(const GURL& x, const GURL& y);

// Equality operator for comparing raw spec_. This should be used in place of
// url == GURL(spec) where |spec| is known (i.e. constants). This is to prevent
// needlessly re-parsing |spec| into a temporary GURL.
COMPONENT_EXPORT(URL)
bool operator==(const GURL& x, std::string_view spec);

// DEBUG_ALIAS_FOR_GURL(var_name, url) copies |url| into a new stack-allocated
// variable named |<var_name>|.  This helps ensure that the value of |url| gets
// preserved in crash dumps.
#define DEBUG_ALIAS_FOR_GURL(var_name, url) \
  DEBUG_ALIAS_FOR_CSTR(var_name, (url).possibly_invalid_spec().c_str(), 128)

namespace url::debug {

class COMPONENT_EXPORT(URL) ScopedUrlCrashKey {
 public:
  ScopedUrlCrashKey(base::debug::CrashKeyString* crash_key, const GURL& value);
  ~ScopedUrlCrashKey();

  ScopedUrlCrashKey(const ScopedUrlCrashKey&) = delete;
  ScopedUrlCrashKey& operator=(const ScopedUrlCrashKey&) = delete;

 private:
  base::debug::ScopedCrashKeyString scoped_string_value_;
};

}  // namespace url::debug

#endif  // URL_GURL_H_
"#;
