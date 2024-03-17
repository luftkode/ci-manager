# CI manager


# Example

```shell
ci-manager \
    --ci=github \
    --verbosity=2 \
        create-issue-from-run \
            --repo=https://github.com/docker/buildx \
            --run-id=8302026485 \
            --title="CI scheduled build" \
            --label=bug \
            --kind=other \
            --trim-timestamp \
            --dry-run
```
