use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilityPermission {
    Granted,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusedElement {
    pub application: String,
    pub role: String,
    pub identifier: Option<String>,
    pub title: Option<String>,
    pub editable: bool,
}

pub trait AccessibilityBackend {
    fn permission(&self) -> Result<AccessibilityPermission, AccessibilityError>;
    fn focused_element(&self) -> Result<Option<FocusedElement>, AccessibilityError>;
    fn insert_text(&self, text: &str) -> Result<(), AccessibilityError>;
}

#[derive(Debug, Default)]
pub struct MacOsAccessibility;

impl AccessibilityBackend for MacOsAccessibility {
    fn permission(&self) -> Result<AccessibilityPermission, AccessibilityError> {
        #[cfg(target_os = "macos")]
        {
            Ok(AccessibilityPermission::Unknown)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(AccessibilityPermission::Denied)
        }
    }

    fn focused_element(&self) -> Result<Option<FocusedElement>, AccessibilityError> {
        Err(AccessibilityError::NotImplemented)
    }

    fn insert_text(&self, _text: &str) -> Result<(), AccessibilityError> {
        Err(AccessibilityError::NotImplemented)
    }
}

#[derive(Debug, Error)]
pub enum AccessibilityError {
    #[error("macOS Accessibility permission is required")]
    PermissionDenied,
    #[error("no editable element is focused")]
    NoEditableElement,
    #[error("accessibility operation is not implemented yet")]
    NotImplemented,
    #[error("accessibility backend failure: {0}")]
    Backend(String),
}
