// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

package dev.accesskit;

import java.nio.charset.StandardCharsets;

@FunctionalInterface
interface NativePointerSupplier {
    long get();
}
