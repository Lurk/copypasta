// Copyright 2016 Avraham Weinstock
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::common::*;
use objc::runtime::{Class, Object, Sel};
use objc::{msg_send, sel, sel_impl};
use objc_foundation::{INSArray, INSString};
use objc_foundation::{NSArray, NSString};
use objc_id::Id;
use std::ffi::CStr;

pub struct OSXClipboardContext {
    pasteboard: Id<Object>,
}

#[allow(non_upper_case_globals)]
static NSUTF8StringEncoding: usize = 4; // apple documentation says it is 4

// required to bring NSPasteboard into the path of the class-resolver
#[link(name = "AppKit", kind = "framework")]
extern "C" {
    pub static NSPasteboardTypeString: Sel;
}

impl OSXClipboardContext {
    pub fn new() -> Result<OSXClipboardContext> {
        let cls = Class::get("NSPasteboard").ok_or("Class::get(\"NSPasteboard\")")?;
        let pasteboard: *mut Object = unsafe { msg_send![cls, generalPasteboard] };
        if pasteboard.is_null() {
            return Err("NSPasteboard#generalPasteboard returned null".into());
        }
        let pasteboard: Id<Object> = unsafe { Id::from_ptr(pasteboard) };
        Ok(OSXClipboardContext { pasteboard })
    }
}

impl ClipboardProvider for OSXClipboardContext {
    fn get_contents(&mut self) -> Result<String> {
        let string: *mut NSString =
            unsafe { msg_send![self.pasteboard, stringForType: NSPasteboardTypeString] };
        if string.is_null() {
            Err("pasteboard#stringForType returned null".into())
        } else {
            let res: String = nsstring_to_rust_string(string).unwrap();
            let _: () = unsafe { msg_send![string, release] };
            Ok(res)
        }
    }

    fn set_contents(&mut self, data: String) -> Result<()> {
        let string_array = NSArray::from_vec(vec![NSString::from_str(&data)]);
        let _: usize = unsafe { msg_send![self.pasteboard, clearContents] };
        let success: bool = unsafe { msg_send![self.pasteboard, writeObjects: string_array] };
        if success {
            Ok(())
        } else {
            Err("NSPasteboard#writeObjects: returned false".into())
        }
    }
}

fn nsstring_to_rust_string(nsstring: *mut NSString) -> Result<String> {
    unsafe {
        let string_size: usize =
            msg_send![nsstring, lengthOfBytesUsingEncoding: NSUTF8StringEncoding];
        // we need +1 because getCString will return null terminated string
        let char_ptr = libc::malloc(string_size + 1);
        let res: bool = msg_send![nsstring, getCString:char_ptr  maxLength:string_size + 1 encoding:NSUTF8StringEncoding];
        if res {
            let c_string = CStr::from_ptr(char_ptr as *const i8);
            libc::free(char_ptr);
            Ok(c_string.to_string_lossy().into_owned())
        } else {
            libc::free(char_ptr);
            Err("Casting from NSString to Rust string has failed".into())
        }
    }
}
