
# Leptos, Axum, Sqlx, tower-sessions, axum_login

This repository ~~will eventually be~~ is a working example of session
management and authentication/authorization using the tools in the title. There
are a few other examples that hit some of those parts, but I want to put it all
together into a simple template, mostly so I could actually learn how it works.
Hopefully it will be helpful to others at some point as well.

## A friendly warning

I'm still getting used to the modern web, so if you notice something crazy
about this repo, you're probably right and I'd love to know about it. This is
the version for leptos 0.6, if you look at the main branch of the repo you'll
see the 0.7 code.

## Getting set up

You should be able to just run `cargo leptos watch` from the project root and
it'll go. There is already a sqlite3 database file with some random user data
in the `db` folder. If you want to log in immediately, go to
[http://127.0.0.1:3000/login](http://127.0.0.1:3000/login) and enter the
username 'asdf' with password 'asdf'.

The only other URL that has anything is the `/register`, which lets you define
a new user. Currently there is **zero feedback** to the user when registration
is successful, so if you click the button and nothing happens, that's normal.
Just go to the `/login` page after that and you'll see that you're logged in as
your new user. 

## Lessons learned in this process

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
- `axum` will let you specify a handler that returns nothing. I assumed at the
  start that this meant all of the request handling was happening through
  side-effects internally (because I was following code from another internet
  source), but that is either wrong or outdated. You need to return stuff. If
  your server functions in leptos are complaining about zero-length payloads
  and failing to deserialize stuff in the browser, this could well be your
  problem.


## Todo list

- [ ] Get this to work with [rauthy](https://gitlab.com/kerkmann/leptos_oidc/-/blob/main/docs/backends/rauthy.md?ref_type=heads) and some other OpenId stuff.
- [ ] Add a profile view to show how to use the sessions for something like a real use-case
- [ ] Make the register page redirect to `/` after success
- [ ] Add a change password page
- [ ] Add at least minimal styling
