FROM alpine:3.13 as bins
ARG PLATFORM
COPY target/output/notifyhealth /app/
RUN apk add binutils && strip /app/notifyhealth

FROM scratch
LABEL maintainer="giggio@giggio.net"
ENTRYPOINT [ "/notifyhealth" ]
COPY --from=bins /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=bins /app/notifyhealth .
