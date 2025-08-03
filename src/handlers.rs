use axum::{extract::Path, response::Html};
use dbus::blocking::Connection;
use log::info;

use crate::{
    dbus_introspection::{
        analyze_service, discover_services, get_service_names_only, introspect_object, ServiceInfo,
    },
    error::{AppError, Result},
    templates::{
        render_dbus_types_reference, render_object_details, render_service_list, PageTemplate,
    },
    utils::{
        build_breadcrumb_navigation, build_object_flat_list, find_child_objects,
        validate_object_path, validate_service_name,
    },
};

pub async fn landing_page() -> Result<Html<String>> {
    info!("Serving landing page");

    let conn = Connection::new_system().map_err(AppError::DbusConnection)?;

    let service_names =
        get_service_names_only(&conn).map_err(|e| AppError::ServiceIntrospection(e.to_string()))?;

    let navigation = r#"<div class="navigation"><a href="/local/dbus_explorer/app">Home</a></div>"#;
    let service_list = render_service_list(&service_names);

    let body = format!("{navigation}{service_list}");
    let page = PageTemplate::new("Home", body);

    Ok(Html(page.render()))
}

pub async fn service_page(Path(service_name): Path<String>) -> Result<Html<String>> {
    let service_name =
        urlencoding::decode(&service_name).map_err(|e| AppError::UrlDecode(e.to_string()))?;

    validate_service_name(&service_name)?;
    info!("Serving service page for: {service_name}");

    let conn = Connection::new_system().map_err(AppError::DbusConnection)?;

    let service_info = analyze_service(&conn, &service_name);

    if service_info.objects.is_empty() && service_info.error.is_some() {
        return Err(AppError::ServiceNotFound(service_name.to_string()));
    }

    let navigation = format!(
        r#"<div class="navigation"><a href="/local/dbus_explorer/app">Home</a> / {}</div>"#,
        html_escape(&service_name)
    );

    let content = render_service_content(&service_info, &service_name);
    let body = format!("{navigation}{content}");

    let page = PageTemplate::new(&service_name, body);
    Ok(Html(page.render()))
}

pub async fn object_page(
    Path((service_name, object_path)): Path<(String, String)>,
) -> Result<Html<String>> {
    let service_name =
        urlencoding::decode(&service_name).map_err(|e| AppError::UrlDecode(e.to_string()))?;
    let object_path =
        urlencoding::decode(&object_path).map_err(|e| AppError::UrlDecode(e.to_string()))?;
    let object_path = format!("/{object_path}");

    validate_service_name(&service_name)?;
    validate_object_path(&object_path)?;

    info!("Serving object page for: {service_name} {object_path}");

    let conn = Connection::new_system().map_err(AppError::DbusConnection)?;

    let object_info = introspect_object(&conn, &service_name, &object_path)
        .ok_or_else(|| AppError::ObjectNotFound(format!("{service_name}:{object_path}")))?;

    // Get all service objects to find children
    let service_info = analyze_service(&conn, &service_name);
    let child_objects = find_child_objects(&service_info.objects, &object_path);

    let navigation = build_breadcrumb_navigation(&service_name, &object_path);
    let object_details = render_object_details(&object_info);
    let child_links = render_child_object_links(&child_objects, &service_name);
    let type_reference = render_dbus_types_reference();

    let body = format!("{navigation}{object_details}{child_links}{type_reference}");
    let title = format!("{service_name} {object_path}");

    let page = PageTemplate::new(&title, body);
    Ok(Html(page.render()))
}

pub async fn all_services_page() -> Result<Html<String>> {
    info!("Serving all services page");

    let conn = Connection::new_system().map_err(AppError::DbusConnection)?;

    let services = discover_services(&conn, None)
        .map_err(|e| AppError::ServiceIntrospection(e.to_string()))?;

    let navigation = r#"<div class="navigation"><a href="/local/dbus_explorer/app">Home</a> / All Services</div>"#;

    let content = render_all_services_content(&services);
    let body = format!("{navigation}{content}");

    let page = PageTemplate::new("All Services and Objects", body);
    Ok(Html(page.render()))
}

fn render_service_content(service_info: &ServiceInfo, service_name: &str) -> String {
    let mut html = String::new();

    // Show service owner information if available
    if let Some(owner) = &service_info.owner {
        html.push_str(&format!(
            r#"<div class="service-info"><strong>Service Owner:</strong> {}</div>"#,
            html_escape(owner)
        ));
    }

    if let Some(error) = &service_info.error {
        html.push_str(&format!(
            r#"<div class="error"><strong>Error:</strong> {}</div>"#,
            html_escape(error)
        ));
        return html;
    }

    html.push_str("<h2>Objects</h2>");
    html.push_str(&build_object_flat_list(&service_info.objects, service_name));

    // Show error objects separately
    let error_objects: Vec<_> = service_info
        .objects
        .iter()
        .filter(|obj| obj.error.is_some())
        .collect();

    if !error_objects.is_empty() {
        html.push_str("<h2>Objects with Errors</h2><ul>");
        for object in error_objects {
            html.push_str(&format!(
                "<li><strong>{}</strong>: {}</li>",
                html_escape(&object.path),
                html_escape(object.error.as_ref().unwrap())
            ));
        }
        html.push_str("</ul>");
    }

    html
}

fn render_all_services_content(services: &[ServiceInfo]) -> String {
    let mut html = String::new();

    for service in services {
        html.push_str(&format!("<h2>Service: {}</h2>", html_escape(&service.name)));

        if let Some(error) = &service.error {
            html.push_str(&format!(
                r#"<div class="error"><strong>Error:</strong> {}</div>"#,
                html_escape(error)
            ));
        } else {
            for object in &service.objects {
                html.push_str(&format!("<h3>Object: {}</h3>", html_escape(&object.path)));
                html.push_str(&render_object_details(object));
            }
        }
    }

    html
}

fn render_child_object_links(
    child_objects: &[&crate::dbus_introspection::ObjectInfo],
    service_name: &str,
) -> String {
    if child_objects.is_empty() {
        return String::new();
    }

    let mut html = String::from("<h2>Child Objects</h2><ul>");

    for child in child_objects {
        // Determine if this object has interfaces
        let has_interfaces = child.interfaces.iter().any(|interface| {
            !interface.methods.is_empty()
                || !interface.properties.is_empty()
                || !interface.signals.is_empty()
        });

        // Create the link
        let url_path = if child.path == "/" {
            String::new()
        } else if child.path.starts_with('/') {
            child.path[1..].to_string()
        } else {
            child.path.clone()
        };

        html.push_str("<li>");

        if has_interfaces {
            // Link to object page if it has interfaces
            html.push_str(&format!(
                r#"<a href="/local/dbus_explorer/app/service/{}/{}">{}</a>"#,
                urlencoding::encode(service_name),
                urlencoding::encode(&url_path),
                html_escape(&child.path)
            ));

            // Show interface count
            let total_interfaces = child.interfaces.len();
            let interfaces_with_content = child
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
                html_escape(&child.path)
            ));
        }

        html.push_str("</li>");
    }

    html.push_str("</ul>");
    html
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
