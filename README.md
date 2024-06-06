# `multiple-headers-bug`

To reprise an Internet Computer bug (headers with duplicate names are removed before sending to
the HTTPS server, what should not happen).

Run test as:
```
docker buildx build -t test2 -f Dockerfile . && docker run test2
```
and watch for assert messages.

Duplicate headers are correctly preserved in the returned result
(as demonostrated by passed first `assert_eq`).