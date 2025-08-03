use std::collections::HashMap;

use crate::{
    dbus_introspection::ObjectInfo,
    error::{AppError, Result},
};

pub fn validate_service_name(service_name: &str) -> Result<()> {
    if service_name.is_empty() {
        return Err(AppError::InvalidInput(
            "Service name cannot be empty".to_string(),
        ));
    }

    if service_name.len() > 255 {
        return Err(AppError::InvalidInput("Service name too long".to_string()));
    }

    // Basic validation for D-Bus service names
    if !service_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
    {
        return Err(AppError::InvalidInput(
            "Invalid characters in service name".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_object_path(object_path: &str) -> Result<()> {
    if object_path.is_empty() {
        return Err(AppError::InvalidInput(
            "Object path cannot be empty".to_string(),
        ));
    }

    if !object_path.starts_with('/') {
        return Err(AppError::InvalidInput(
            "Object path must start with '/'".to_string(),
        ));
    }

    if object_path.len() > 1024 {
        return Err(AppError::InvalidInput("Object path too long".to_string()));
    }

    // Basic validation for D-Bus object paths
    if !object_path
        .chars()
        .all(|c| c.is_alphanumeric() || c == '/' || c == '_' || c == '-')
    {
        return Err(AppError::InvalidInput(
            "Invalid characters in object path".to_string(),
        ));
    }

    Ok(())
}

pub fn build_breadcrumb_navigation(service_name: &str, object_path: &str) -> String {
    let mut breadcrumb_links = Vec::new();

    // Add home link
    breadcrumb_links.push(r#"<a href="/local/dbus_explorer/app">Home</a>"#.to_string());

    // Add service link
    breadcrumb_links.push(format!(
        r#"<a href="/local/dbus_explorer/app/service/{}">{}</a>"#,
        urlencoding::encode(service_name),
        html_escape(service_name)
    ));

    // Build object path breadcrumbs
    if object_path != "/" {
        let parts: Vec<&str> = object_path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_path = String::new();

        for (i, part) in parts.iter().enumerate() {
            current_path.push('/');
            current_path.push_str(part);

            if i == parts.len() - 1 {
                // Last part - no link
                breadcrumb_links.push(html_escape(part));
            } else {
                // Intermediate parts - create links
                let url_path = current_path[1..].to_string();
                breadcrumb_links.push(format!(
                    r#"<a href="/local/dbus_explorer/app/service/{}/{}">{}</a>"#,
                    urlencoding::encode(service_name),
                    urlencoding::encode(&url_path),
                    html_escape(part)
                ));
            }
        }
    } else {
        breadcrumb_links.push("/".to_string());
    }

    format!(
        r#"<div class="navigation">{}</div>"#,
        breadcrumb_links.join(" / ")
    )
}

pub fn build_object_flat_list(objects: &[ObjectInfo], service_name: &str) -> String {
    // Deduplicate objects by path
    let mut unique_objects = HashMap::new();
    for object in objects {
        unique_objects.insert(&object.path, object);
    }

    let mut html = String::from("<ul>");

    // Create sorted list of objects
    let mut sorted_objects: Vec<&ObjectInfo> = unique_objects.values().copied().collect();
    sorted_objects.sort_by(|a, b| a.path.cmp(&b.path));

    for object in sorted_objects {
        if object.error.is_some() {
            continue;
        }

        // Determine if this object has interfaces
        let has_interfaces = object.interfaces.iter().any(|interface| {
            !interface.methods.is_empty()
                || !interface.properties.is_empty()
                || !interface.signals.is_empty()
        });

        // Create the link
        let url_path = if object.path == "/" {
            String::new()
        } else if object.path.starts_with('/') {
            object.path[1..].to_string()
        } else {
            object.path.clone()
        };

        html.push_str("<li>");

        if has_interfaces {
            // Link to object page if it has interfaces
            html.push_str(&format!(
                r#"<a href="/local/dbus_explorer/app/service/{}/{}">{}</a>"#,
                urlencoding::encode(service_name),
                urlencoding::encode(&url_path),
                html_escape(&object.path)
            ));

            // Show interface count
            let total_interfaces = object.interfaces.len();
            let interfaces_with_content = object
                .interfaces
                .iter()
                .filter(|interface| {
                    !interface.methods.is_empty()
                        || !interface.properties.is_empty()
                        || !interface.signals.is_empty()
                })
                .count();

            if total_interfaces > 0 {
                if interfaces_with_content == total_interfaces {
                    // All interfaces have content
                    html.push_str(&format!(
                        " <em>({} interface{})</em>",
                        total_interfaces,
                        if total_interfaces == 1 { "" } else { "s" }
                    ));
                } else if interfaces_with_content > 0 {
                    // Some interfaces have content
                    html.push_str(&format!(
                        " <em>({} of {} interface{} with content)</em>",
                        interfaces_with_content,
                        total_interfaces,
                        if total_interfaces == 1 { "" } else { "s" }
                    ));
                } else {
                    // No interfaces have content
                    html.push_str(&format!(
                        " <em>({} interface{}, no content)</em>",
                        total_interfaces,
                        if total_interfaces == 1 { "" } else { "s" }
                    ));
                }
            }
        } else {
            // Just show name for navigation-only objects
            html.push_str(&format!(
                "<strong>{}</strong> <em>(navigation only)</em>",
                html_escape(&object.path)
            ));
        }

        html.push_str("</li>");
    }

    html.push_str("</ul>");
    html
}

pub fn find_child_objects<'a>(objects: &'a [ObjectInfo], parent_path: &str) -> Vec<&'a ObjectInfo> {
    let mut children = Vec::new();

    for object in objects {
        if object.error.is_some() {
            continue;
        }

        // Check if this object is a direct child of the parent path
        if object.path != parent_path && object.path.starts_with(parent_path) {
            let remaining_path = if parent_path == "/" {
                &object.path[1..]
            } else {
                &object.path[parent_path.len()..]
            };

            // If remaining path starts with '/' and contains no more '/', it's a direct child
            if remaining_path.starts_with('/') && !remaining_path[1..].contains('/') {
                children.push(object);
            }
        }
    }

    children.sort_by(|a, b| a.path.cmp(&b.path));
    children
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
