# podcast/fetch.rs

HTTP(S) feed/metadata downloads (8 MB limit) and streaming episode downloads (4 GB limit).
Bound connection/transfer duration and redirect count; preserve normal TLS validation. No caller
authorization header is forwarded to a feed host. Error messages omit URLs because private feed
URLs may contain credentials. Local/self-hosted feeds are permitted, so paired devices must be
trusted to request feed imports. Never supports file URLs, shell execution or filesystem URLs.
