FROM alpine:3 as bins
FROM scratch
LABEL maintainer="giggio@giggio.net"
ENTRYPOINT [ "/notifyhealth" ]
COPY --from=bins /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY target/output/notifyhealth /
