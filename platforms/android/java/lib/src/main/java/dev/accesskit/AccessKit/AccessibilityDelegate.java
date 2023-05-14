/**
 * Copyright 2023 The AccessKit Authors. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (found in
 * the LICENSE-APACHE file) or the MIT license (found in
 * the LICENSE-MIT file), at your option.
 */

package dev.accesskit.AccessKit;

import android.os.Bundle;
import android.view.accessibility.AccessibilityNodeProvider;
import android.view.View;
import androidx.annotation.Nullable;
import androidx.core.view.accessibility.AccessibilityNodeInfoCompat;
import androidx.core.view.accessibility.AccessibilityNodeProviderCompat;
import androidx.core.view.AccessibilityDelegateCompat;

public final class AccessibilityDelegate extends View.AccessibilityDelegate {
    private AccessibilityNodeProviderCompat provider;
    private long context;
    
    @Override
    public AccessibilityNodeProvider getAccessibilityNodeProvider(View host) {
        if (this.provider == null) {
            this.provider = new AccessibilityNodeProviderCompat() {
                @Override
                @Nullable
                public AccessibilityNodeInfoCompat createAccessibilityNodeInfo(int virtualViewId) {
                    AccessibilityNodeInfoCompat node = null;
                    if (virtualViewId == AccessibilityNodeProviderCompat.HOST_VIEW_ID) {
                        node = AccessibilityNodeInfoCompat.obtain(host);
                        onInitializeAccessibilityNodeInfo(host, node.unwrap());
                    } else {
                        node = AccessibilityNodeInfoCompat.obtain(host, virtualViewId);
                        node.setPackageName(host.getContext().getPackageName());
                    }
                    populateAccessibilityNodeInfo(context, host, node, virtualViewId);
                    return node;
                }

                @Override
                public boolean performAction(int virtualViewId, int action, Bundle arguments) {
                    if (virtualViewId == AccessibilityNodeProviderCompat.HOST_VIEW_ID)
                        return host.performAccessibilityAction(action, arguments);
                    return false;
                }
            };
        }
        
        return (AccessibilityNodeProvider)this.provider.getProvider();
    }
    
    private static native AccessibilityNodeInfoCompat populateAccessibilityNodeInfo(long context, View host, AccessibilityNodeInfoCompat node, int virtualViewId);
}
