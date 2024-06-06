# `multiple-headers-bug`

To reprise an Internet Computer bug (headers with duplicate names are removed before sending to
the HTTPS server, what should not happen).

Run test as:
```
docker buildx build -t test2 -f Dockerfile . && docker run test2
```
and watch for error messages.

This test does not check whether duplicate headers are (erroneously) removed from returned
result.