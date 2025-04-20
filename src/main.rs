#![allow(unexpected_cfgs)]
use core_foundation::base::{CFGetTypeID, CFRelease};
use core_graphics::geometry::CGRect;
use std::ffi::{c_void};
use core_foundation::base::{CFTypeRef};
use core_foundation::string::{CFString};
use std::ptr::null_mut;
use accessibility_sys::{kAXFocusedWindowAttribute, kAXValueTypeCGRect, AXUIElementCopyAttributeValue, AXUIElementCreateApplication, AXUIElementRef, AXValueGetValue, AXValueRef};
use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy, NSBackingStoreBuffered, NSColor, NSWindow, NSWindowStyleMask};
use cocoa::base::{id, nil};
use cocoa::foundation::{ NSPoint, NSRect, NSSize};
use objc2_app_kit::{NSWorkspace};
use core_foundation::base::{TCFType};
use objc::runtime::{Class, Object};

fn get_frontmost_pid() -> Option<i32> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let pid = workspace.frontmostApplication()?.processIdentifier();
        Some(pid)
    }
}

fn get_window_frame(window: AXUIElementRef) -> Option<CGRect> {
    unsafe {
        let mut focused_window_ref: CFTypeRef = null_mut();
        let result = AXUIElementCopyAttributeValue(
            window,
            CFString::new(kAXFocusedWindowAttribute).as_concrete_TypeRef(),
            &mut focused_window_ref,
        );
        if result == 0 && !focused_window_ref.is_null() {
            let focused_window: AXUIElementRef = focused_window_ref as AXUIElementRef;
            let mut cf_value: CFTypeRef = null_mut();
            let result = AXUIElementCopyAttributeValue(
                focused_window,
                CFString::new("AXFrame").as_concrete_TypeRef(),
                &mut cf_value,
            );
            let mut rect = CGRect::default();
            let success = AXValueGetValue(
                cf_value as AXValueRef,
                kAXValueTypeCGRect,
                &mut rect as *mut _ as *mut c_void
            );
            // Todo probably other pointers lack
            CFRelease(cf_value);
            CFRelease(focused_window_ref);
            return if success {
                Some(rect)
            } else {
                None
            }
        }
        None
    }
}

fn main() {
    if let Some(pid) = get_frontmost_pid() {
        println!("Frontmost pid is {}", pid);
        unsafe {
            let app = AXUIElementCreateApplication(pid);
            match get_window_frame(app) {
                Some(screen_frame) => {
                    println!("Screen frame is {:?}", screen_frame);
                    let app = NSApp();
                    app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);
                    app.finishLaunching();
                    let style_mask = NSWindowStyleMask::NSBorderlessWindowMask;
                    let frame = NSRect::new(
                        NSPoint::new(screen_frame.origin.x, screen_frame.origin.y),
                        NSSize::new(screen_frame.size.width, screen_frame.size.height),
                    );
                    let window: id = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                        frame,
                        style_mask,
                        NSBackingStoreBuffered,
                        false,
                    );
                    let cls: *const Class = Class::get("NSColor").expect("NSColor class not found");
                    let cls_ptr: id = cls as *const Object as id;
                    let red_color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
                       cls_ptr,
                        1.0,
                        0.0,
                        0.0,
                        0.4,
                    );
                    window.setLevel_(26);
                    window.setOpaque_(false);
                    window.setBackgroundColor_(NSColor::clearColor(nil));
                    window.setBackgroundColor_(red_color);
                    window.setIgnoresMouseEvents_(true);
                    window.makeKeyAndOrderFront_(nil);
                    app.run();

                }
                None => {print!("Something wrong occured when retrieving the coordinates...")}
            }
        }
    } else {
        println!("Impossible de récupérer le PID de l'application active.");
    }
}