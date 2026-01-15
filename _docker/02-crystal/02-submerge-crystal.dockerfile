ARG version
FROM helikon/submerge-lib:$version as builder

FROM helikon/submerge-base:$version
# copy executable
COPY --from=builder /submerge/bin/submerge-crystal /usr/local/bin/
CMD ["submerge-crystal"]