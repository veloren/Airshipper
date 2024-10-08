use axum::response::Html;

pub async fn index() -> Html<&'static str> {
    Html(
        r#"<html>
<h1>Veloren Airshipper Download Server</h1>

You can find the <a href="https://veloren.net/download">airshipper client here</a><br>

Check for supported channels via /channels/&lt;os&gt;/&lt;arch&gt; :<br>
<ul>
 <li><a href="/channels/linux/x86_64">/channels/linux/x86_64</a></li>
 <li><a href="/channels/windows/x86_64">/channels/windows/x86_64</a></li>
 <li><a href="/channels/macos/x86_64">/channels/macos/x86_64</a></li>
 <li><a href="/channels/linux/aarch64">/channels/linux/aarch64</a></li>
 <li><a href="/channels/macos/aarch64">/channels/macos/aarch64</a></li>
</ul>

Check for new versions via /version/&lt;os&gt;/&lt;arch&gt;/&lt;channel&gt; :<br>
<ul>
 <li><a href="/version/linux/x86_64/weekly">/version/linux/x86_64/weekly</a></li>
 <li><a href="/version/linux/x86_64/nightly">/version/linux/x86_64/nightly</a></li>
 <li><a href="/version/windows/x86_64/weekly">/version/windows/x86_64/weekly</a></li>
 <li><a href="/version/windows/x86_64/nightly">/version/windows/x86_64/nightly</a></li>
</ul>

Manually download new versions via /latest/&lt;os&gt;/&lt;arch&gt;/&lt;channel&gt; :<br>
<ul>
 <li><a href="/latest/linux/x86_64/nightly">/latest/linux/x86_64/nightly</a></li>
 <li><a href="/latest/windows/x86_64/nightly">/latest/windows/x86_64/nightly</a></li>
</ul>

Read public service announcement: <a href="/announcement">/announcement</a> <br>
</html>"#,
    )
}

pub async fn robots() -> &'static str {
    "User-agent: *
     Disallow: /"
}
