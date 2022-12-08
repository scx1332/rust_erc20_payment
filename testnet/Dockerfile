FROM ethereum/client-go as builder

FROM python:3.11.0-bullseye
RUN apt-get update
RUN apt-get install -y vim

COPY --from=builder /usr/local/bin/geth /usr/local/bin/

WORKDIR /runtime
COPY *.py .