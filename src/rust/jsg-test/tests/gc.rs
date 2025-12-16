//! GC (garbage collection) tests for Rust resources.
//!
//! These tests verify that Rust resources are properly cleaned up when:
//! 1. All Rust `Ref` handles are dropped and no JavaScript wrapper exists
//! 2. All Rust `Ref` handles are dropped and V8 garbage collects the JS wrapper

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use jsg::Lock;
use jsg::Resource;
use jsg::Type;
use jsg::v8::TracedReference;
use jsg_macros::jsg_method;
use jsg_macros::jsg_resource;

use crate::ffi;

/// Counter to track how many `SimpleResource` instances have been dropped.
static SIMPLE_RESOURCE_DROPS: AtomicUsize = AtomicUsize::new(0);

#[jsg_resource]
struct SimpleResource {
    pub name: String,
    pub callback: Option<TracedReference<jsg::v8::Object>>,
}

impl Drop for SimpleResource {
    fn drop(&mut self) {
        SIMPLE_RESOURCE_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[jsg_resource]
#[expect(clippy::unnecessary_wraps)]
impl SimpleResource {
    #[jsg_method]
    fn get_name(&self) -> Result<String, String> {
        Ok(self.name.clone())
    }
}

/// Counter to track how many `ParentResource` instances have been dropped.
static PARENT_RESOURCE_DROPS: AtomicUsize = AtomicUsize::new(0);

#[jsg_resource]
struct ParentResource {
    pub child: jsg::Ref<SimpleResource>,
    pub optional_child: Option<jsg::Ref<SimpleResource>>,
}

impl Drop for ParentResource {
    fn drop(&mut self) {
        PARENT_RESOURCE_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[jsg_resource]
impl ParentResource {}

/// Tests that resources are dropped when all Rust Refs are dropped and no JS wrapper exists.
///
/// When a resource is allocated but never wrapped for JavaScript, dropping all `Ref` handles
/// should immediately deallocate the resource.
#[test]
fn supports_gc_via_realm_drop() {
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let resource = SimpleResource::alloc(
            &mut lock,
            SimpleResource {
                name: "test".to_owned(),
                callback: None,
            },
        );
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
        std::mem::drop(resource);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    });
}

/// Tests that resources are dropped via V8 GC weak callback when JS wrapper is collected.
///
/// When a resource is wrapped for JavaScript:
/// 1. Dropping all Rust `Ref` handles makes the V8 Global weak
/// 2. V8 GC can then collect the wrapper and trigger the weak callback
/// 3. The weak callback deallocates the resource
#[test]
fn supports_gc_via_weak_callback() {
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let resource = SimpleResource::alloc(
            &mut lock,
            SimpleResource {
                name: "test".to_owned(),
                callback: None,
            },
        );
        let _wrapped = SimpleResource::wrap(resource.clone(), &mut lock);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
        std::mem::drop(resource);
        // There is a JS object that holds a reference to the resource
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
    });

    harness.run_in_context(|isolate, _ctx| unsafe {
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
        ffi::request_gc(isolate);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn resource_with_ref_field_compiles() {
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);
    PARENT_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let child = SimpleResource {
            name: "child".to_owned(),
            callback: None,
        };
        let child_ref = SimpleResource::alloc(&mut lock, child);

        let optional_child = SimpleResource {
            name: "optional_child".to_owned(),
            callback: None,
        };
        let optional_child_ref = SimpleResource::alloc(&mut lock, optional_child);

        let parent = ParentResource {
            child: child_ref,
            optional_child: Some(optional_child_ref),
        };
        let parent_ref = ParentResource::alloc(&mut lock, parent);
        let _wrapped = ParentResource::wrap(parent_ref.clone(), &mut lock);

        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
        assert_eq!(PARENT_RESOURCE_DROPS.load(Ordering::SeqCst), 0);

        std::mem::drop(parent_ref);
    });

    harness.run_in_context(|isolate, _ctx| unsafe {
        ffi::request_gc(isolate);
        assert_eq!(PARENT_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn child_ref_kept_alive_by_parent() {
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);
    PARENT_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let child = SimpleResource {
            name: "child".to_owned(),
            callback: None,
        };
        let child_ref = SimpleResource::alloc(&mut lock, child);
        let child_ref_clone = child_ref.clone();

        let parent = ParentResource {
            child: child_ref,
            optional_child: None,
        };
        let parent_ref = ParentResource::alloc(&mut lock, parent);
        let _parent_wrapped = ParentResource::wrap(parent_ref.clone(), &mut lock);

        std::mem::drop(child_ref_clone);

        // Child not collected because parent still holds a reference
        ffi::request_gc(isolate);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);

        std::mem::drop(parent_ref);

        // Now both are collected
        ffi::request_gc(isolate);
        assert_eq!(PARENT_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn weak_ref_upgrade() {
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let resource = SimpleResource {
            name: "test".to_owned(),
            callback: None,
        };
        let strong_ref = SimpleResource::alloc(&mut lock, resource);
        let weak_ref = jsg::WeakRef::from(&strong_ref);

        // Weak ref can be upgraded while strong ref exists
        assert_eq!(weak_ref.strong_count(), 1);
        let upgraded = weak_ref.upgrade();
        assert!(upgraded.is_some());
        assert_eq!(weak_ref.strong_count(), 2);

        // Drop the upgraded ref
        std::mem::drop(upgraded);
        assert_eq!(weak_ref.strong_count(), 1);

        // Drop the original strong ref
        std::mem::drop(strong_ref);

        // Resource should be dropped now
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    });
}

// ============================================================================
// cppgc module tests
// ============================================================================

#[test]
fn cppgc_handle_keeps_resource_alive() {
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let resource = SimpleResource {
            name: "test".to_owned(),
            callback: None,
        };
        let strong_ref = SimpleResource::alloc(&mut lock, resource);

        // Wrap the resource to allocate it on the cppgc heap
        let _wrapped = SimpleResource::wrap(strong_ref.clone(), &mut lock);

        // Drop the Rust ref - resource should still be alive due to cppgc handle
        std::mem::drop(strong_ref);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
    });

    // After context exits and GC runs, resource should be dropped
    harness.run_in_context(|isolate, _ctx| unsafe {
        ffi::request_gc(isolate);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn cppgc_weak_member_cleared_after_gc() {
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let resource = SimpleResource {
            name: "test".to_owned(),
            callback: None,
        };
        let strong_ref = SimpleResource::alloc(&mut lock, resource);

        // Wrap to create cppgc allocation, then create a weak ref
        let _wrapped = SimpleResource::wrap(strong_ref.clone(), &mut lock);
        let weak_ref = jsg::WeakRef::from(&strong_ref);

        // Weak ref should be upgradeable while strong ref exists
        assert!(weak_ref.upgrade().is_some());

        // Drop strong ref and wrapped object goes out of scope
        std::mem::drop(strong_ref);

        // Resource not dropped yet - JS wrapper holds it
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
    });

    // After GC, weak member should be cleared
    harness.run_in_context(|isolate, _ctx| unsafe {
        ffi::request_gc(isolate);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn cppgc_weak_handle_default_behavior() {
    // WeakHandle doesn't have a Default impl, but we can verify it
    // doesn't prevent GC when the strong handle is released
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let resource = SimpleResource {
            name: "test".to_owned(),
            callback: None,
        };
        let strong_ref = SimpleResource::alloc(&mut lock, resource);

        // Wrap the resource
        let _wrapped = SimpleResource::wrap(strong_ref.clone(), &mut lock);

        // Drop Rust ref - cppgc handle still holds it
        std::mem::drop(strong_ref);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
    });

    // GC should collect it
    harness.run_in_context(|isolate, _ctx| unsafe {
        ffi::request_gc(isolate);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    });
}

/// Test that traced weak references are properly managed during GC
#[test]
fn cppgc_weak_member_traced_in_gc() {
    SIMPLE_RESOURCE_DROPS.store(0, Ordering::SeqCst);
    PARENT_RESOURCE_DROPS.store(0, Ordering::SeqCst);

    let harness = crate::Harness::new();
    harness.run_in_context(|isolate, _ctx| unsafe {
        let mut lock = Lock::from_isolate_ptr(isolate);
        let child = SimpleResource {
            name: "child".to_owned(),
            callback: None,
        };
        let child_ref = SimpleResource::alloc(&mut lock, child);

        // Wrap child to create cppgc allocation
        let _child_wrapped = SimpleResource::wrap(child_ref.clone(), &mut lock);

        let parent = ParentResource {
            child: child_ref.clone(),
            optional_child: None,
        };
        let parent_ref = ParentResource::alloc(&mut lock, parent);

        // Wrap parent
        let _parent_wrapped = ParentResource::wrap(parent_ref.clone(), &mut lock);

        // Drop Rust refs
        std::mem::drop(child_ref);
        std::mem::drop(parent_ref);

        // Nothing dropped yet - JS wrappers keep them alive
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
        assert_eq!(PARENT_RESOURCE_DROPS.load(Ordering::SeqCst), 0);
    });

    // GC may take multiple cycles to collect both resources
    // Parent holds a Ref to child, so child won't be collected until parent is
    harness.run_in_context(|isolate, _ctx| unsafe {
        // First GC - should collect parent (no strong refs)
        ffi::request_gc(isolate);
        // Parent dropped, which releases the child Ref
        assert_eq!(PARENT_RESOURCE_DROPS.load(Ordering::SeqCst), 1);

        // Second GC - should collect child now that parent released the Ref
        ffi::request_gc(isolate);
        assert_eq!(SIMPLE_RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    });
}
