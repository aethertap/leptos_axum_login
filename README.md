
# Leptos, Axum, Sqlx, tower-sessions, axum_login

This repository will eventually be a working example of session management and
authentication/authorization using the tools in the title. There are a few
other examples that hit some of those parts, but I want to put it all together
into a simple template, mostly so I could actually learn how it works.
Hopefully it will be helpful to others at some point as well.

## A friendly warning

I'm an old-school developer from back in the bad old days before a lot of this technology was even an idea, so
I am coming at this with a lot of antiquated notions! I'm trying to relearn the modern web and stick to best
practices, but if you notice something crazy about this repo, you're probably right.

I would *not* just copy and paste bits and pieces of this code into your own project at this time, although
I intend to get feedback on it and bring it to a state where that should be just fine. At the moment, you should
definitely have a skeptical eye, although it does at least *actually work* with the latest versions of things.

## Lessons learned in this process

- `leptos_axum` wraps the `Session` from `tower-sessions` but you can still access it directly with `leptos_axum::extract()`
- Be careful to keep the `session_auth_hash` and the stored password hash in the database separate in your mind. I
  had a long debugging session wherein I mixed them together somewhat randomly, and it was horrible.
    - If you find that your sessions are invalidated every time you switch pages in your app, it is probably because
    there is something wrong with your `session_auth_hash`. Look there first.
- You can't have an empty session that gets an id to the browser. If there's no data in the session, it won't send an id, period.
  This took me significant headscratching to realize, because my "simplest possible case to see if the session layer was inserted correctly" was doing nothing.
- Calling `auth_session.login(user).await` *does* trigger the session to be sent to the browser, you *don't* have to do a `session.save().await`
- Don't forget to `.await` on everything involving sessions and auth. This got me for a while as it gave no compiler warnings.
- `axum` will let you specify a handler that returns nothing. I assumed at the start that this meant all of the
  request handling was happening through side-effects internally (because I was following code from another internet source),
  but that is wrong. You need to return stuff. If your server functions in leptos are complaining about zero-length payloads and failing to
  deserialize stuff, this could well be your problem.
