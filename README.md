
# Leptos, Axum, Sqlx, tower-sessions, axum_login

This repository is a working example of session management and
authentication/authorization using the tools in the title. There are a few
other examples that hit some of those parts, but I want to put it all together
into a simple template, mostly so I could actually learn how it works.
Hopefully it will be helpful to others at some point as well.

## Getting set up

You should be able to just run `cargo leptos watch` from the project root and
it'll go. There is already a sqlite3 database file with some random user data
in the `db` folder. If you want to log in immediately, go to
[http://127.0.0.1:3000/login](http://127.0.0.1:3000/login) and enter the
username 'asdf' with password 'asdf'.

The "home" page will redirect you to the login page if you aren't logged in,
mostly to demo how that process works (at least how it works if you do it the
way I did. There might be better ways).

The only other URL that has anything is the `/register`, which lets you define
a new user. Currently there is **zero feedback** to the user when registration
is successful, so if you click the button and nothing happens, that's normal.
Just go to the `/login` page after that and you'll see that you're logged in as
your new user. 

## Lessons learned in this process

- If you get `wasm-bindgen` version number problems, the solution has two
  steps. First, install the latest versions of `cargo-leptos` and
  `wasm-bindgen-cli` (`cargo install cargo-leptos wasm-bindgen-cli`). Then,
  update the `wasm-bindgen` version in your `Cargo.toml` to the latest version
  (which will be helpfully provided by the error message).
- `axum_login::AuthSession` wraps the `Session` from `tower-sessions` but you
  can still access it directly with `leptos_axum::extract()`
- Be careful to keep the `session_auth_hash` and the stored password hash in
  the database separate in your mind. I had a long debugging session wherein I
  mixed them together somewhat randomly, and it was horrible.
    - If you find that your sessions are invalidated every time you switch
      pages in your app, it is probably because there is something wrong with
      your `session_auth_hash`. Look there first.
- You can't have an empty session that gets an id to the browser. If there's no
  data in the session, it won't send an id, period. This took me significant
  headscratching to realize (though it is stated in the documentation!), because
  my "simplest possible case to see if the session layer was inserted
  correctly" was doing nothing.
- Calling `auth_session.login(user).await` *does* trigger the session to be
  sent to the browser, you *don't* have to do a `session.save().await`
- Don't forget to `.await` on everything involving sessions and auth. This got
  me for a while as it gave no compiler warnings.
- `axum` allows you to specify a handler that returns nothing. I assumed at the
  start that this meant all of the request handling was happening through
  side-effects internally (because I was following code from another internet
  source), but that is either wrong or outdated. You need to return stuff. If
  your server functions in leptos are complaining about zero-length payloads
  and failing to deserialize stuff in the browser, this could well be your
  problem.


## Todo list

- [ ] Get this to work with [rauthy](https://gitlab.com/kerkmann/leptos_oidc/-/blob/main/docs/backends/rauthy.md?ref_type=heads) and some other OpenId stuff.
- [ ] Add a profile view to show how to use the sessions for something like a real use-case
- [x] Make the register page redirect to `/` after success
- [ ] Add a change password page
- [x] Add at least minimal styling
- [x] port the code to the latest leptos (0.7rc3 as of last time I updated the README)
- [ ] Make a version of this that works as a progressive web app that can be installed to run offline

## Update log


- 2024-12-17: Updated to leptos 0.7.1, wasm-bindgen 0.2.99
- 2025-01-24: cargo update; build check

