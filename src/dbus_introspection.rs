use std::time::Duration;

use anyhow::{Context, Result};
use dbus::blocking::Connection;
use log::debug;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub owner: Option<String>,
    pub objects: Vec<ObjectInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub path: String,
    pub interfaces: Vec<InterfaceInfo>,
    pub error: Option<String>,
    pub child_nodes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub name: String,
    pub methods: Vec<MethodInfo>,
    pub properties: Vec<PropertyInfo>,
    pub signals: Vec<SignalInfo>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub arguments: Vec<ArgumentInfo>,
    pub return_values: Vec<ArgumentInfo>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: String,
    pub type_name: String,
    pub access: String, // "read", "write", or "readwrite"
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SignalInfo {
    pub name: String,
    pub arguments: Vec<ArgumentInfo>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArgumentInfo {
    pub name: Option<String>,
    pub type_name: String,
    #[allow(dead_code)]
    pub direction: Option<String>, // "in" or "out"
    #[allow(dead_code)]
    pub description: Option<String>,
}

// Serde structs for D-Bus introspection XML parsing
#[derive(Debug, Deserialize)]
struct DbusNode {
    #[serde(rename = "interface", default)]
    interfaces: Vec<DbusInterface>,
    #[serde(rename = "node", default)]
    child_nodes: Vec<DbusChildNode>,
}

#[derive(Debug, Deserialize)]
struct DbusInterface {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "method", default)]
    methods: Vec<DbusMethod>,
    #[serde(rename = "property", default)]
    properties: Vec<DbusProperty>,
    #[serde(rename = "signal", default)]
    signals: Vec<DbusSignal>,
    #[serde(rename = "annotation", default)]
    annotations: Vec<DbusAnnotation>,
}

#[derive(Debug, Deserialize)]
struct DbusMethod {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "arg", default)]
    arguments: Vec<DbusArg>,
    #[serde(rename = "annotation", default)]
    annotations: Vec<DbusAnnotation>,
}

#[derive(Debug, Deserialize)]
struct DbusProperty {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type")]
    type_name: String,
    #[serde(rename = "@access")]
    access: String,
    #[serde(rename = "annotation", default)]
    _annotations: Vec<DbusAnnotation>,
}

#[derive(Debug, Deserialize)]
struct DbusSignal {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "arg", default)]
    arguments: Vec<DbusArg>,
    #[serde(rename = "annotation", default)]
    annotations: Vec<DbusAnnotation>,
}

#[derive(Debug, Deserialize)]
struct DbusArg {
    #[serde(rename = "@name")]
    name: Option<String>,
    #[serde(rename = "@type")]
    type_name: String,
    #[serde(rename = "@direction")]
    direction: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DbusAnnotation {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct DbusChildNode {
    #[serde(rename = "@name")]
    name: String,
}

pub fn get_service_names_only(conn: &Connection) -> Result<Vec<String>> {
    let proxy = conn.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(1000), // Reduced timeout
    );

    let (names,): (Vec<String>,) = proxy
        .method_call("org.freedesktop.DBus", "ListNames", ())
        .context("Failed to list D-Bus names")?;

    let mut service_names = Vec::new();
    for name in names {
        if !name.starts_with(':') {
            service_names.push(name);
        }
    }

    service_names.sort();
    Ok(service_names)
}

pub fn discover_services(conn: &Connection, filter: Option<&str>) -> Result<Vec<ServiceInfo>> {
    let proxy = conn.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(2000), // Reduced timeout
    );

    let (names,): (Vec<String>,) = proxy
        .method_call("org.freedesktop.DBus", "ListNames", ())
        .context("Failed to list D-Bus names")?;

    let mut services = Vec::new();

    for name in names {
        if name.starts_with(':') {
            continue;
        }

        if let Some(filter_pattern) = filter {
            if !name.contains(filter_pattern) {
                continue;
            }
        }

        let service_info = analyze_service(conn, &name);
        services.push(service_info);
    }

    services.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(services)
}

pub fn analyze_service(conn: &Connection, service_name: &str) -> ServiceInfo {
    let mut service_info = ServiceInfo {
        name: service_name.to_string(),
        owner: None,
        objects: Vec::new(),
        error: None,
    };

    let dbus_proxy = conn.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(500),
    );

    if let Ok((owner,)) = dbus_proxy.method_call::<(String,), _, _, _>(
        "org.freedesktop.DBus",
        "GetNameOwner",
        (service_name,),
    ) {
        service_info.owner = Some(owner);
    }

    // Only start from root path - no guessing, always record results
    let root_object = introspect_object(conn, service_name, "/");

    if let Some(root_obj) = root_object {
        // Check if root has error or is accessible
        if root_obj.error.is_none() {
            // Root is accessible, recursively explore all discoverable paths
            let mut objects_to_explore = vec![root_obj];

            while let Some(current_object) = objects_to_explore.pop() {
                // Add child nodes to exploration queue
                for child_node in &current_object.child_nodes {
                    let child_path = if current_object.path == "/" {
                        format!("/{child_node}")
                    } else {
                        format!("{}/{}", current_object.path, child_node)
                    };

                    // Always attempt introspection and store result (including errors)
                    if let Some(child_object) = introspect_object(conn, service_name, &child_path) {
                        objects_to_explore.push(child_object);
                    }
                }

                service_info.objects.push(current_object);
            }
        } else {
            // Root has error, but still record it to show what happened
            service_info.objects.push(root_obj);
        }
    } else {
        // This should not happen since introspect_object always returns Some
        service_info.error = Some("Failed to attempt root introspection".to_string());
    }

    if service_info.objects.is_empty() && service_info.error.is_none() {
        service_info.error =
            Some("No accessible objects found or service not authorized".to_string());
    }

    service_info
}

pub fn introspect_object(
    conn: &Connection,
    service_name: &str,
    object_path: &str,
) -> Option<ObjectInfo> {
    let proxy = conn.with_proxy(service_name, object_path, Duration::from_millis(1000)); // Reduced timeout

    match proxy.method_call::<(String,), _, _, _>(
        "org.freedesktop.DBus.Introspectable",
        "Introspect",
        (),
    ) {
        Ok((xml,)) => match parse_introspection_xml_serde(&xml, service_name, object_path) {
            Ok((interfaces, child_nodes)) => Some(ObjectInfo {
                path: object_path.to_string(),
                interfaces,
                error: None,
                child_nodes,
            }),
            Err(e) => Some(ObjectInfo {
                path: object_path.to_string(),
                interfaces: Vec::new(),
                error: Some(format!("XML parsing failed: {e}")),
                child_nodes: Vec::new(),
            }),
        },
        Err(e) => {
            let error_msg = if e
                .to_string()
                .contains("org.freedesktop.DBus.Error.AccessDenied")
            {
                "Access denied - not authorized to introspect this object".to_string()
            } else if e
                .to_string()
                .contains("org.freedesktop.DBus.Error.UnknownMethod")
            {
                "Object does not support introspection".to_string()
            } else {
                format!("Introspection failed: {e}")
            };

            Some(ObjectInfo {
                path: object_path.to_string(),
                interfaces: Vec::new(),
                error: Some(error_msg),
                child_nodes: Vec::new(),
            })
        }
    }
}

fn log_xml_document(service_name: &str, object_path: &str, xml: &str, success: bool) {
    if success {
        debug!("XML parsing successful for {service_name}:{object_path}\nContent:\n{xml}");
    } else {
        debug!("XML parsing failed for {service_name}:{object_path}\nContent:\n{xml}");
    }
}

fn parse_introspection_xml_serde(
    xml: &str,
    service_name: &str,
    object_path: &str,
) -> Result<(Vec<InterfaceInfo>, Vec<String>)> {
    // Parse the XML document
    let parse_result = quick_xml::de::from_str::<DbusNode>(xml);

    let dbus_node = match parse_result {
        Ok(node) => {
            // Log successful parses at debug level
            log_xml_document(service_name, object_path, xml, true);
            node
        }
        Err(e) => {
            // Log failed parses for non-freedesktop services to reduce noise
            if !service_name.starts_with("org.freedesktop.") {
                log_xml_document(service_name, object_path, xml, false);
            }
            return Err(anyhow::anyhow!(
                "Failed to parse D-Bus introspection XML for {}:{}: {}",
                service_name,
                object_path,
                e
            ));
        }
    };

    let mut interfaces = Vec::new();
    let mut child_nodes = Vec::new();

    // Extract child node names
    for child_node in dbus_node.child_nodes {
        child_nodes.push(child_node.name);
    }

    // Convert D-Bus interfaces to our internal format - show all interfaces
    for dbus_interface in dbus_node.interfaces {
        let mut interface = InterfaceInfo {
            name: dbus_interface.name,
            methods: Vec::new(),
            properties: Vec::new(),
            signals: Vec::new(),
            description: None,
        };

        // Find interface description from annotations
        for annotation in &dbus_interface.annotations {
            if annotation.name == "org.freedesktop.DBus.Description" {
                interface.description = Some(annotation.value.clone());
                break;
            }
        }

        // Convert methods
        for dbus_method in dbus_interface.methods {
            let mut method = MethodInfo {
                name: dbus_method.name,
                arguments: Vec::new(),
                return_values: Vec::new(),
                description: None,
            };

            // Find method description from annotations
            for annotation in &dbus_method.annotations {
                if annotation.name == "org.freedesktop.DBus.Description" {
                    method.description = Some(annotation.value.clone());
                    break;
                }
            }

            // Convert arguments
            for dbus_arg in dbus_method.arguments {
                let arg = ArgumentInfo {
                    name: dbus_arg.name,
                    type_name: dbus_arg.type_name,
                    direction: dbus_arg.direction.clone(),
                    description: None,
                };

                if dbus_arg.direction.as_deref() == Some("out") {
                    method.return_values.push(arg);
                } else {
                    method.arguments.push(arg);
                }
            }

            interface.methods.push(method);
        }

        // Convert properties
        for dbus_property in dbus_interface.properties {
            let property = PropertyInfo {
                name: dbus_property.name,
                type_name: dbus_property.type_name,
                access: dbus_property.access,
                description: None,
            };
            interface.properties.push(property);
        }

        // Convert signals
        for dbus_signal in dbus_interface.signals {
            let mut signal = SignalInfo {
                name: dbus_signal.name,
                arguments: Vec::new(),
                description: None,
            };

            // Find signal description from annotations
            for annotation in &dbus_signal.annotations {
                if annotation.name == "org.freedesktop.DBus.Description" {
                    signal.description = Some(annotation.value.clone());
                    break;
                }
            }

            // Convert signal arguments
            for dbus_arg in dbus_signal.arguments {
                let arg = ArgumentInfo {
                    name: dbus_arg.name,
                    type_name: dbus_arg.type_name,
                    direction: dbus_arg.direction,
                    description: None,
                };
                signal.arguments.push(arg);
            }

            interface.signals.push(signal);
        }

        interfaces.push(interface);
    }

    Ok((interfaces, child_nodes))
}
