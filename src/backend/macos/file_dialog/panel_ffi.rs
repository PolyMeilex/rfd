use core::str;
use std::cell::Cell;
use std::path::Path;
use std::path::PathBuf;

use block2::Block;
use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol};
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSModalResponse, NSOpenPanel, NSPopUpButton, NSSavePanel,
    NSTextField, NSView, NSWindow, NSWindowLevel,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString, NSURL};
use objc2_uniform_type_identifiers::UTType;
use raw_window_handle::RawWindowHandle;

use super::super::{
    modal_future::{AsModal, InnerModal},
    utils::{FocusManager, PolicyManager},
};
use super::save_name;
use crate::backend::macos::utils::window_from_raw_window_handle;
use crate::file_dialog::Filter;
use crate::FileDialog;

extern "C" {
    pub fn CGShieldingWindowLevel() -> i32;
}

pub struct Panel {
    // Either `NSSavePanel` or the subclass `NSOpenPanel`
    pub(crate) panel: Retained<NSSavePanel>,
    parent: Option<Retained<NSWindow>>,
    _focus_manager: FocusManager,
    _policy_manager: PolicyManager,
    // NSControl.target is an unretained (assign) reference, so this is the
    // only thing keeping the save panel's format-picker handler alive.
    _save_panel_handler: Option<Retained<SavePanelHandler>>,
}

impl AsModal for Panel {
    fn inner_modal(&self) -> &(impl InnerModal + 'static) {
        &*self.panel
    }
}

impl InnerModal for NSSavePanel {
    fn begin_modal(&self, window: &NSWindow, handler: &Block<dyn Fn(NSModalResponse)>) {
        unsafe { self.beginSheetModalForWindow_completionHandler(window, handler) }
    }

    fn run_modal(&self) -> NSModalResponse {
        unsafe { self.runModal() }
    }
}

impl Panel {
    fn as_open_panel(&self) -> Option<&NSOpenPanel> {
        self.panel.downcast_ref::<NSOpenPanel>()
    }

    pub fn new(panel: Retained<NSSavePanel>, parent: Option<&RawWindowHandle>) -> Self {
        let _policy_manager = PolicyManager::new(MainThreadMarker::from(&*panel));

        let _focus_manager = FocusManager::new(MainThreadMarker::from(&*panel));

        panel.setLevel(unsafe { CGShieldingWindowLevel() } as NSWindowLevel);
        Self {
            panel,
            parent: parent.map(window_from_raw_window_handle),
            _focus_manager,
            _policy_manager,
            _save_panel_handler: None,
        }
    }

    pub fn run_modal(&self) -> NSModalResponse {
        if let Some(parent) = self.parent.clone() {
            let completion = block2::StackBlock::new(|_: isize| {});

            unsafe {
                self.panel
                    .beginSheetModalForWindow_completionHandler(&parent, &completion)
            }
        }

        unsafe { self.panel.runModal() }
    }

    pub fn get_result(&self) -> PathBuf {
        unsafe {
            let url = self.panel.URL().unwrap();
            url.path().unwrap().to_string().into()
        }
    }

    /// The file a save panel chose, with the extension of the selected format.
    pub fn get_save_result(&self) -> PathBuf {
        let path = self.get_result();
        match &self._save_panel_handler {
            Some(handler) => handler.path_with_format_extension(path),
            None => path,
        }
    }

    pub fn get_results(&self) -> Vec<PathBuf> {
        unsafe {
            let urls = self.as_open_panel().unwrap().URLs();

            let mut res = Vec::new();
            for url in urls {
                res.push(url.path().unwrap().to_string().into());
            }

            res
        }
    }
}

fn utype_for_extension(ext: &str) -> Option<Retained<UTType>> {
    unsafe { UTType::typeWithFilenameExtension(&NSString::from_str(ext)) }
}

fn utypes_for_filter(filter: &Filter) -> Retained<NSArray<UTType>> {
    let types: Vec<_> = filter
        .extensions
        .iter()
        .filter_map(|ext| utype_for_extension(ext))
        .collect();

    NSArray::from_retained_slice(&types)
}

trait PanelExt {
    fn panel(&self) -> &NSSavePanel;

    fn set_can_create_directories(&self, can: bool) {
        unsafe { self.panel().setCanCreateDirectories(can) }
    }

    fn set_shows_hidden_files(&self, show: bool) {
        unsafe { self.panel().setShowsHiddenFiles(show) }
    }

    fn add_filters(&self, opt: &FileDialog) {
        let mut exts: Vec<String> = Vec::new();

        for filter in opt.filters.iter() {
            exts.append(&mut filter.extensions.to_vec());
        }

        let f_raw: Vec<_> = exts.iter().map(|ext| NSString::from_str(ext)).collect();
        let array = NSArray::from_retained_slice(&f_raw);

        unsafe {
            #[allow(deprecated)]
            self.panel().setAllowedFileTypes(Some(&array));
        }
    }

    fn set_path(&self, path: &Path) {
        // directoryURL must point at a directory; a starting file name is
        // applied separately through `setNameFieldStringValue`.
        if let Some(path) = path.to_str() {
            unsafe {
                let url = NSURL::fileURLWithPath_isDirectory(&NSString::from_str(path), true);
                self.panel().setDirectoryURL(Some(&url));
            }
        }
    }

    fn set_file_name(&self, file_name: &str) {
        unsafe {
            self.panel()
                .setNameFieldStringValue(&NSString::from_str(file_name))
        }
    }

    fn set_title(&self, title: &str) {
        unsafe { self.panel().setMessage(Some(&NSString::from_str(title))) }
    }
}

impl PanelExt for Retained<NSSavePanel> {
    fn panel(&self) -> &NSSavePanel {
        &self
    }
}

impl PanelExt for Retained<NSOpenPanel> {
    fn panel(&self) -> &NSSavePanel {
        &self
    }
}

struct FormatOption {
    types: Retained<NSArray<UTType>>,
    extensions: Vec<String>,
    // Whether the panel can be given `types`, see `save_name::format_is_exact`. When it can't, the
    // panel would change the extension of the name to the one its type prefers, so it is left to
    // accept any name.
    exact: bool,
}

impl FormatOption {
    fn new(filter: &Filter) -> Self {
        let exact = save_name::format_is_exact(&filter.extensions, |ext| {
            let preferred = unsafe { utype_for_extension(ext)?.preferredFilenameExtension() };
            preferred.map(|preferred| preferred.to_string())
        });
        Self {
            types: utypes_for_filter(filter),
            extensions: filter.extensions.clone(),
            exact,
        }
    }
}

/// Gives the panel what the format needs: its types when it is exact, and when it is not, every
/// type, with other file types allowed. `allows_other_file_types` is the panel's own setting.
fn apply_format(panel: &NSSavePanel, option: &FormatOption, allows_other_file_types: bool) {
    if option.exact {
        unsafe {
            panel.setAllowedContentTypes(&option.types);
            panel.setAllowsOtherFileTypes(allows_other_file_types);
        }
    } else {
        let any: Vec<Retained<UTType>> = Vec::new();
        unsafe {
            panel.setAllowedContentTypes(&NSArray::from_retained_slice(&any));
            panel.setAllowsOtherFileTypes(true);
        }
    }
}

struct SavePanelHandlerIvars {
    panel: Retained<NSSavePanel>,
    options: Vec<FormatOption>,
    // The name the panel was opened with, see `FileDialog::set_file_name`
    file_name: Option<String>,
    // Index in `options` of the selected format
    selected: Cell<usize>,
    // What `allowsOtherFileTypes` was before a format changed it
    allows_other_file_types: bool,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = SavePanelHandlerIvars]
    struct SavePanelHandler;

    unsafe impl NSObjectProtocol for SavePanelHandler {}

    impl SavePanelHandler {
        #[unsafe(method(formatChanged:))]
        fn format_changed(&self, sender: &NSPopUpButton) {
            let idx = unsafe { sender.indexOfSelectedItem() };
            if idx < 0 {
                return;
            }

            let ivars = self.ivars();
            let Some(option) = ivars.options.get(idx as usize) else {
                return;
            };
            let previous = ivars.selected.replace(idx as usize);

            // The name is read before the allowed types change, and set after them, so the panel
            // never sees a name whose extension is not allowed.
            let name = unsafe { ivars.panel.nameFieldStringValue() }.to_string();
            let previous_extensions = ivars
                .options
                .get(previous)
                .map_or(&[][..], |previous| &previous.extensions);
            let renamed = save_name::name_for_format(
                &name,
                previous_extensions,
                ivars.file_name.as_deref(),
                &option.extensions,
            );

            apply_format(&ivars.panel, option, ivars.allows_other_file_types);

            if let Some(renamed) = renamed {
                unsafe {
                    ivars
                        .panel
                        .setNameFieldStringValue(&NSString::from_str(&renamed))
                };
            }
        }
    }
);

impl SavePanelHandler {
    fn new(
        mtm: MainThreadMarker,
        panel: Retained<NSSavePanel>,
        options: Vec<FormatOption>,
        file_name: Option<String>,
    ) -> Retained<Self> {
        let allows_other_file_types = unsafe { panel.allowsOtherFileTypes() };
        let this = Self::alloc(mtm).set_ivars(SavePanelHandlerIvars {
            panel,
            options,
            file_name,
            selected: Cell::new(0),
            allows_other_file_types,
        });
        unsafe { msg_send![super(this), init] }
    }

    /// Gives the panel what the first format needs, the one that is selected when it opens.
    fn apply_first_format(&self) {
        let ivars = self.ivars();
        if let Some(first) = ivars.options.first() {
            apply_format(&ivars.panel, first, ivars.allows_other_file_types);
        }
    }

    /// `path`, a name the panel returned, with the first extension of the selected format when the
    /// format is not exact and the name has none. The panel returns the name as it was typed then.
    /// A name that would be another file's is left as the panel returned it, since the panel only
    /// asked about overwriting that one.
    fn path_with_format_extension(&self, path: PathBuf) -> PathBuf {
        let ivars = self.ivars();
        let Some(option) = ivars.options.get(ivars.selected.get()) else {
            return path;
        };
        if option.exact {
            return path;
        }
        let renamed = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| save_name::name_with_format_extension(name, &option.extensions))
            .map(|name| path.with_file_name(name));
        match renamed {
            Some(renamed) if !renamed.exists() => renamed,
            _ => path,
        }
    }
}

fn build_format_accessory_view(
    mtm: MainThreadMarker,
    filters: &[Filter],
    label_text: &str,
) -> (Retained<NSView>, Retained<NSPopUpButton>) {
    // NSSavePanel doesn't add its own padding around the accessory view, so the
    // view's height needs enough slack above/below the controls (once centered)
    // to avoid feeling cramped against the file list and the button row.
    let view_height = 44.0;
    let popup_width = 220.0;
    let popup_height = 25.0;

    // `labelWithString` already sizes the field to fit, but `sizeToFit` makes that
    // explicit and lets us read back the real (font-metric-based) size below,
    // rather than guessing a pixel width from the string's byte length.
    let label = unsafe { NSTextField::labelWithString(&NSString::from_str(label_text), mtm) };
    unsafe { label.sizeToFit() };
    let label_size = label.frame().size;

    let popup_x = label_size.width + 5.0;

    let view = unsafe {
        NSView::initWithFrame(
            NSView::alloc(mtm),
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(popup_x + popup_width, view_height),
            ),
        )
    };
    // Keeps the row centered (rather than pinned to its initial absolute position)
    // if the user resizes the save panel, without stretching it: `ViewWidthSizable`
    // would make NSSavePanel drop the standard margin it gives a fixed-size
    // accessory view, jamming the row against the panel's left edge.
    unsafe {
        view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewMinXMargin | NSAutoresizingMaskOptions::ViewMaxXMargin,
        )
    };

    unsafe {
        label.setFrameOrigin(NSPoint::new(0.0, (view_height - label_size.height) / 2.0));
    }

    let popup = unsafe { NSPopUpButton::new(mtm) };
    unsafe {
        popup.setFrame(NSRect::new(
            NSPoint::new(popup_x, (view_height - popup_height) / 2.0),
            NSSize::new(popup_width, popup_height),
        ))
    };
    for filter in filters {
        unsafe { popup.addItemWithTitle(&NSString::from_str(&filter.name)) };
    }

    unsafe {
        view.addSubview(&label);
        view.addSubview(&popup);
    }

    (view, popup)
}

impl Panel {
    pub fn build_pick_file(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        if !opt.filters.is_empty() {
            panel.add_filters(opt);
        }

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        if let Some(file_name) = &opt.file_name {
            panel.set_file_name(file_name);
        }

        if let Some(title) = &opt.title {
            panel.set_title(title);
        }

        if let Some(can) = opt.can_create_directories {
            panel.set_can_create_directories(can);
        }

        if let Some(show) = opt.show_hidden_files {
            panel.set_shows_hidden_files(show);
        }

        unsafe { panel.setCanChooseDirectories(false) };
        unsafe { panel.setCanChooseFiles(true) };

        Self::new(Retained::into_super(panel), opt.parent.as_ref())
    }

    pub fn build_save_file(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSSavePanel::savePanel(mtm) };

        let save_panel_handler = match opt.filters.as_slice() {
            [] => None,
            filters => {
                let options = filters.iter().map(FormatOption::new).collect();
                let handler =
                    SavePanelHandler::new(mtm, panel.clone(), options, opt.file_name.clone());
                handler.apply_first_format();

                if filters.len() > 1 {
                    let label_text = opt.format_label.as_deref().unwrap_or("Format:");
                    let (view, popup) = build_format_accessory_view(mtm, filters, label_text);

                    unsafe {
                        popup.setTarget(Some(&handler));
                        popup.setAction(Some(sel!(formatChanged:)));
                    }

                    unsafe {
                        popup.selectItemAtIndex(0);
                        panel.setAccessoryView(Some(&view));
                    }
                }

                Some(handler)
            }
        };

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        if let Some(file_name) = &opt.file_name {
            panel.set_file_name(file_name);
        }

        if let Some(title) = &opt.title {
            panel.set_title(title);
        }

        if let Some(can) = opt.can_create_directories {
            panel.set_can_create_directories(can);
        }

        if let Some(show) = opt.show_hidden_files {
            panel.set_shows_hidden_files(show);
        }

        let mut result = Self::new(panel, opt.parent.as_ref());
        result._save_panel_handler = save_panel_handler;
        result
    }

    pub fn build_pick_folder(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        if let Some(title) = &opt.title {
            panel.set_title(title);
        }

        let can = opt.can_create_directories.unwrap_or(true);
        panel.set_can_create_directories(can);

        if let Some(show) = opt.show_hidden_files {
            panel.set_shows_hidden_files(show);
        }

        unsafe { panel.setCanChooseDirectories(true) };
        unsafe { panel.setCanChooseFiles(false) };

        Self::new(Retained::into_super(panel), opt.parent.as_ref())
    }

    pub fn build_pick_folders(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        if let Some(title) = &opt.title {
            panel.set_title(title);
        }

        let can = opt.can_create_directories.unwrap_or(true);
        panel.set_can_create_directories(can);

        if let Some(show) = opt.show_hidden_files {
            panel.set_shows_hidden_files(show);
        }

        unsafe { panel.setCanChooseDirectories(true) };
        unsafe { panel.setCanChooseFiles(false) };
        unsafe { panel.setAllowsMultipleSelection(true) };

        Self::new(Retained::into_super(panel), opt.parent.as_ref())
    }

    pub fn build_pick_files(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        if !opt.filters.is_empty() {
            panel.add_filters(opt);
        }

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        if let Some(title) = &opt.title {
            panel.set_title(title);
        }

        if let Some(can) = opt.can_create_directories {
            panel.set_can_create_directories(can);
        }

        if let Some(show) = opt.show_hidden_files {
            panel.set_shows_hidden_files(show);
        }

        unsafe { panel.setCanChooseDirectories(false) };
        unsafe { panel.setCanChooseFiles(true) };
        unsafe { panel.setAllowsMultipleSelection(true) };

        Self::new(Retained::into_super(panel), opt.parent.as_ref())
    }

    pub fn build_pick_file_or_folder(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        if !opt.filters.is_empty() {
            panel.add_filters(opt);
        }

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        if let Some(title) = &opt.title {
            panel.set_title(title);
        }

        let can = opt.can_create_directories.unwrap_or(true);
        panel.set_can_create_directories(can);

        if let Some(show) = opt.show_hidden_files {
            panel.set_shows_hidden_files(show);
        }

        unsafe { panel.setCanChooseDirectories(true) };
        unsafe { panel.setCanChooseFiles(true) };
        unsafe { panel.setAllowsMultipleSelection(false) };

        Self::new(Retained::into_super(panel), opt.parent.as_ref())
    }

    pub fn build_pick_files_or_folders(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        if !opt.filters.is_empty() {
            panel.add_filters(opt);
        }

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        if let Some(title) = &opt.title {
            panel.set_title(title);
        }

        let can = opt.can_create_directories.unwrap_or(true);
        panel.set_can_create_directories(can);

        if let Some(show) = opt.show_hidden_files {
            panel.set_shows_hidden_files(show);
        }

        unsafe { panel.setCanChooseDirectories(true) };
        unsafe { panel.setCanChooseFiles(true) };
        unsafe { panel.setAllowsMultipleSelection(true) };

        Self::new(Retained::into_super(panel), opt.parent.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact(extensions: &[&str]) -> bool {
        FormatOption::new(&Filter {
            name: "Format".to_string(),
            extensions: extensions.iter().map(|ext| ext.to_string()).collect(),
        })
        .exact
    }

    /// The types the system itself declares for these extensions prefer them.
    #[test]
    fn formats_of_the_systems_own_types_are_exact() {
        assert!(exact(&["png"]));
        assert!(exact(&["pdf"]));
    }

    #[test]
    fn a_format_that_accepts_any_name_is_not_exact() {
        assert!(!exact(&["*"]));
        assert!(!exact(&[""]));
        assert!(!exact(&[]));
    }
}
