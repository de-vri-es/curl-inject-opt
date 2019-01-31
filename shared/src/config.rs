// Copyright 2018-2019 Maarten de Vries <maarten@de-vri.es>
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::path::Path;

mod raw {
	include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

pub fn prefix() -> &'static Path {
	Path::new(raw::PREFIX)
}

pub fn libdir() -> &'static Path {
	Path::new(raw::LIBDIR_RESOLVED)
}

pub fn bindir() -> &'static Path {
	Path::new(raw::BINDIR_RESOLVED)
}

pub fn datadir() -> &'static Path {
	Path::new(raw::DATADIR_RESOLVED)
}

pub fn rely_on_search() -> bool {
	raw::RELY_ON_SEARCH
}

pub fn prefix_raw() -> &'static str {
	raw::PREFIX
}

pub fn libdir_raw() -> &'static str {
	raw::LIBDIR
}

pub fn bindir_raw() -> &'static str {
	raw::BINDIR
}

pub fn datadir_raw() -> &'static str {
	raw::DATADIR
}
