use crate::dbus_introspection::ObjectInfo;

pub struct PageTemplate {
    pub title: String,
    pub body: String,
}

impl PageTemplate {
    pub fn new(title: &str, body: String) -> Self {
        Self {
            title: title.to_string(),
            body,
        }
    }

    pub fn render(self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>D-Bus Explorer - {}</title>
    <style>
        body {{ font-family: 'Courier New', 'Monaco', 'Menlo', monospace; margin: 20px; }}
        table {{ font-family: 'Courier New', 'Monaco', 'Menlo', monospace; border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        code {{ background-color: #f0f0f0; padding: 2px 4px; border-radius: 3px; }}
        .navigation {{ margin-bottom: 20px; padding: 10px; background-color: #f8f9fa; border-radius: 4px; }}
        .error {{ color: #d32f2f; background: #ffebee; padding: 15px; border-radius: 4px; margin: 10px 0; }}
        .interface {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 4px; }}
        .method, .property, .signal {{ margin: 10px 0; padding: 8px; background-color: #f8f9fa; border-radius: 3px; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            self.title, self.title, self.body
        )
    }
}

pub fn render_service_list(service_names: &[String]) -> String {
    let mut html = String::from(
        r#"
<h2>Services</h2>
<ul>
"#,
    );

    for service_name in service_names {
        html.push_str(&format!(
            r#"    <li><a href="/local/dbus_explorer/app/service/{}">{}</a></li>
"#,
            urlencoding::encode(service_name),
            html_escape(service_name)
        ));
    }

    html.push_str(
        r#"</ul>
<h2>All Services and Objects</h2>
<p><a href="/local/dbus_explorer/app/all">View all services and objects (flattened)</a></p>
"#,
    );

    html
}

pub fn render_object_details(object: &ObjectInfo) -> String {
    let mut html = String::new();

    if let Some(error) = &object.error {
        html.push_str(&format!(
            r#"<div class="error"><strong>Error:</strong> {}</div>"#,
            html_escape(error)
        ));
        return html;
    }

    for interface in &object.interfaces {
        html.push_str(&format!(
            r#"<div class="interface">
<h4>Interface: {}</h4>"#,
            html_escape(&interface.name)
        ));

        if let Some(desc) = &interface.description {
            html.push_str(&format!(r#"<p><em>{}</em></p>"#, html_escape(desc)));
        }

        // Check if interface is empty
        let is_empty = interface.methods.is_empty()
            && interface.properties.is_empty()
            && interface.signals.is_empty();

        if is_empty {
            html.push_str(r#"<p><em>No exposed methods, properties, or signals</em></p>"#);
        }

        // Methods
        if !interface.methods.is_empty() {
            html.push_str("<h5>Methods:</h5>");
            for method in &interface.methods {
                html.push_str(&format!(
                    r#"<div class="method">
<strong>{}({})</strong>"#,
                    html_escape(&method.name),
                    method
                        .arguments
                        .iter()
                        .map(|arg| format!(
                            "{}: {}",
                            arg.name.as_deref().unwrap_or("_"),
                            &arg.type_name
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));

                if !method.return_values.is_empty() {
                    let returns: Vec<String> = method
                        .return_values
                        .iter()
                        .map(|ret| {
                            format!("{}: {}", ret.name.as_deref().unwrap_or("_"), &ret.type_name)
                        })
                        .collect();
                    html.push_str(&format!(" → {}", returns.join(", ")));
                }

                if let Some(desc) = &method.description {
                    html.push_str(&format!("<br><em>{}</em>", html_escape(desc)));
                }
                html.push_str("</div>");
            }
        }

        // Properties
        if !interface.properties.is_empty() {
            html.push_str("<h5>Properties:</h5>");
            for property in &interface.properties {
                html.push_str(&format!(
                    r#"<div class="property">
<strong>{}</strong> {} [{}]"#,
                    html_escape(&property.name),
                    html_escape(&property.type_name),
                    html_escape(&property.access)
                ));

                if let Some(desc) = &property.description {
                    html.push_str(&format!("<br><em>{}</em>", html_escape(desc)));
                }
                html.push_str("</div>");
            }
        }

        // Signals
        if !interface.signals.is_empty() {
            html.push_str("<h5>Signals:</h5>");
            for signal in &interface.signals {
                html.push_str(&format!(
                    r#"<div class="signal">
<strong>{}({})</strong>"#,
                    html_escape(&signal.name),
                    signal
                        .arguments
                        .iter()
                        .map(|arg| format!(
                            "{}: {}",
                            arg.name.as_deref().unwrap_or("_"),
                            &arg.type_name
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));

                if let Some(desc) = &signal.description {
                    html.push_str(&format!("<br><em>{}</em>", html_escape(desc)));
                }
                html.push_str("</div>");
            }
        }

        html.push_str("</div>");
    }

    html
}

pub fn render_dbus_types_reference() -> String {
    r#"
<hr>
<h3>D-Bus Type Reference</h3>
<table>
<tr><th>Type</th><th>Description</th><th>Example</th></tr>
<tr><td><code>b</code></td><td>Boolean</td><td>true, false</td></tr>
<tr><td><code>y</code></td><td>Byte (8-bit unsigned)</td><td>255</td></tr>
<tr><td><code>n</code></td><td>Int16 (16-bit signed)</td><td>-32768</td></tr>
<tr><td><code>q</code></td><td>UInt16 (16-bit unsigned)</td><td>65535</td></tr>
<tr><td><code>i</code></td><td>Int32 (32-bit signed)</td><td>-2147483648</td></tr>
<tr><td><code>u</code></td><td>UInt32 (32-bit unsigned)</td><td>4294967295</td></tr>
<tr><td><code>x</code></td><td>Int64 (64-bit signed)</td><td>-9223372036854775808</td></tr>
<tr><td><code>t</code></td><td>UInt64 (64-bit unsigned)</td><td>18446744073709551615</td></tr>
<tr><td><code>d</code></td><td>Double (IEEE 754)</td><td>3.14159</td></tr>
<tr><td><code>s</code></td><td>String (UTF-8)</td><td>"Hello World"</td></tr>
<tr><td><code>o</code></td><td>Object path</td><td>/com/example/Object</td></tr>
<tr><td><code>g</code></td><td>Signature</td><td>s, i, as</td></tr>
<tr><td><code>v</code></td><td>Variant (any type)</td><td>Any of the above</td></tr>
<tr><td><code>a</code></td><td>Array prefix</td><td><code>as</code> = array of strings</td></tr>
<tr><td><code>()</code></td><td>Struct</td><td><code>(si)</code> = struct with string and int</td></tr>
<tr><td><code>{}</code></td><td>Dictionary entry</td><td><code>a{sv}</code> = array of string-variant pairs</td></tr>
</table>
<p><em>Common patterns: <code>as</code> = string array, <code>a{sv}</code> = property map, <code>a(ssss)</code> = array of 4-string structs</em></p>
"#.to_string()
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
