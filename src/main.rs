//! The web binary.
//!
//! Everything that turns this checkout into a running application is in
//! `src/lib.rs`, so that an optional native package can call it too. What is
//! left here is the part only a web deployment has: the public address, and a
//! terminal to report to.

use upurnaka::{db, initialize_application, rahti, routes};

#[tokio::main]
async fn main() {
    let app = match initialize_application().await {
        Ok(app) => app,
        Err(e) => {
            // A terminal wants exactly this. A window has nowhere to print,
            // which is why the error is returned rather than printed by the
            // function that produced it.
            eprintln!("rahti: the application could not start ({}): {e}", e.step);
            std::process::exit(1);
        }
    };

    // `cargo run -- --migrate` applies pending migrations and stops.
    //
    // Deliberate, and behind an argument, because a server that changes a
    // schema because somebody restarted it is a surprise — and the
    // production version of that surprise is much worse. This is the branch
    // docs/conventions/database.md asks for: one command, run by a person
    // who meant to run it.
    if std::env::args().any(|argument| argument == "--migrate") {
        if !db::is_connected() {
            eprintln!("rahti: no database is configured — set DATABASE_URL first.");
            std::process::exit(1);
        }

        match db::migrate().await {
            Ok(()) => println!("Migrations applied."),
            Err(error) => {
                eprintln!("rahti: {error}");
                std::process::exit(1);
            }
        }

        rahti::jobs::finish_shutdown().await;
        return;
    }

    // Static export runs this same startup and router without binding a
    // listener, so build-time HTML cannot drift from the live application.
    if let Some(output) = rahti::static_output() {
        match rahti::export_static(
            app.router,
            routes::static_routes().await,
            rahti::public_dir(routes::PUBLIC_DIR),
            output,
        )
        .await
        {
            Ok(report) => {
                report.print();
                rahti::jobs::finish_shutdown().await;
                return;
            }
            Err(error) => {
                eprintln!("rahti: static export failed: {error}");
                std::process::exit(1);
            }
        }
    }

    // The configured address, unless the environment names another: a
    // deployment sets `PORT` (and `HOST` if it needs one), so this line is
    // the same in development and in production. In dev mode a busy port
    // falls forward to the next free one — the line below names the address
    // that actually bound. In release a busy port is an error: whatever
    // routes traffic to this process points at the port it was given.
    let listener = rahti::listen(app.host, app.port).await;

    // Not the bind address verbatim: `0.0.0.0` is every interface, which is
    // not somewhere a browser can go.
    println!("Server running on {}", rahti::local_url(&listener));

    // Attach the transport peer to every request. The rate limiter uses this
    // address by default and ignores spoofable forwarding headers.
    axum::serve(
        listener,
        app.router
            .into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        rahti::shutdown_signal().await;
        println!("\nShutting down...");
    })
    .await
    .unwrap();

    // HTTP has stopped accepting work. Finish jobs already accepted during
    // the request drain, bounded by each queue's shutdown grace period.
    rahti::jobs::finish_shutdown().await;
    println!("Server stopped.");
}
