package dev.accesskit.AccessKit;

import android.os.Bundle;
import android.view.View;
import androidx.annotation.Nullable;
import androidx.core.view.accessibility.AccessibilityNodeInfoCompat;
import androidx.core.view.accessibility.AccessibilityNodeProviderCompat;
import androidx.core.view.AccessibilityDelegateCompat;

public final class AccessibilityDelegate extends AccessibilityDelegateCompat {
    private AccessibilityNodeProviderCompat provider;
    private long ptr;
    
    @Override
    public AccessibilityNodeProviderCompat getAccessibilityNodeProvider(View host) {
        if (this.provider == null) {
            this.provider = new AccessibilityNodeProviderCompat() {
                @Override
                @Nullable
                public AccessibilityNodeInfoCompat createAccessibilityNodeInfo(int virtualViewId) {
                    AccessibilityNodeInfoCompat node = null;
                    if (virtualViewId == AccessibilityNodeProviderCompat.HOST_VIEW_ID) {
                        node = AccessibilityNodeInfoCompat.obtain(host);
                        onInitializeAccessibilityNodeInfo(host, node);
                    } else {
                        node = AccessibilityNodeInfoCompat.obtain(host, virtualViewId);
                        node.setPackageName(host.getContext().getPackageName());
                    }
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
        return this.provider;
    }
}
