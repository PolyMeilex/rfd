use core::str;
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
    // only thing keeping the panel's format-picker handler alive.
    _format_handler: Option<Retained<FormatHandler>>,
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
            _format_handler: None,
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
}

struct FormatHandlerIvars {
    panel: Retained<NSSavePanel>,
    options: Vec<FormatOption>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = FormatHandlerIvars]
    struct FormatHandler;

    unsafe impl NSObjectProtocol for FormatHandler {}

    impl FormatHandler {
        #[unsafe(method(formatChanged:))]
        fn format_changed(&self, sender: &NSPopUpButton) {
            let idx = unsafe { sender.indexOfSelectedItem() };
            if idx < 0 {
                return;
            }

            if let Some(option) = self.ivars().options.get(idx as usize) {
                unsafe { self.ivars().panel.setAllowedContentTypes(&option.types) };
            }
        }
    }
);

impl FormatHandler {
    fn new(
        mtm: MainThreadMarker,
        panel: Retained<NSSavePanel>,
        options: Vec<FormatOption>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(FormatHandlerIvars { panel, options });
        unsafe { msg_send![super(this), init] }
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

fn add_format_picker(
    mtm: MainThreadMarker,
    panel: &Retained<NSSavePanel>,
    filters: &[Filter],
    opt: &FileDialog,
) -> Retained<FormatHandler> {
    let label_text = opt.format_label.as_deref().unwrap_or("Format:");
    let (view, popup) = build_format_accessory_view(mtm, filters, label_text);

    let options = filters
        .iter()
        .map(|filter| FormatOption {
            types: utypes_for_filter(filter),
        })
        .collect();

    let handler = FormatHandler::new(mtm, panel.clone(), options);

    unsafe {
        popup.setTarget(Some(&handler));
        popup.setAction(Some(sel!(formatChanged:)));
    }

    unsafe {
        panel.setAllowedContentTypes(&utypes_for_filter(&filters[0]));
        popup.selectItemAtIndex(0);
        panel.setAccessoryView(Some(&view));
    }

    handler
}

fn add_open_filters(
    mtm: MainThreadMarker,
    panel: &Retained<NSOpenPanel>,
    opt: &FileDialog,
) -> Option<Retained<FormatHandler>> {
    match opt.filters.as_slice() {
        [] => None,
        [_] => {
            panel.add_filters(opt);
            None
        }
        multiple => {
            let panel = Retained::into_super(panel.clone());
            Some(add_format_picker(mtm, &panel, multiple, opt))
        }
    }
}

impl Panel {
    pub fn build_pick_file(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        let format_handler = add_open_filters(mtm, &panel, opt);

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

        let mut result = Self::new(Retained::into_super(panel), opt.parent.as_ref());
        result._format_handler = format_handler;
        result
    }

    pub fn build_save_file(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSSavePanel::savePanel(mtm) };

        let format_handler = match opt.filters.as_slice() {
            [] => None,
            [single] => {
                unsafe { panel.setAllowedContentTypes(&utypes_for_filter(single)) };
                None
            }
            multiple => Some(add_format_picker(mtm, &panel, multiple, opt)),
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
        result._format_handler = format_handler;
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

        let format_handler = add_open_filters(mtm, &panel, opt);

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

        let mut result = Self::new(Retained::into_super(panel), opt.parent.as_ref());
        result._format_handler = format_handler;
        result
    }

    pub fn build_pick_file_or_folder(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        let format_handler = add_open_filters(mtm, &panel, opt);

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

        let mut result = Self::new(Retained::into_super(panel), opt.parent.as_ref());
        result._format_handler = format_handler;
        result
    }

    pub fn build_pick_files_or_folders(opt: &FileDialog, mtm: MainThreadMarker) -> Self {
        let panel = unsafe { NSOpenPanel::openPanel(mtm) };

        let format_handler = add_open_filters(mtm, &panel, opt);

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

        let mut result = Self::new(Retained::into_super(panel), opt.parent.as_ref());
        result._format_handler = format_handler;
        result
    }
}
