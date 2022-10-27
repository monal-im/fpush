# Fpush

`fpush` is a scalable push server for XMPP implemeting[XEP-0357](https://xmpp.org/extensions/xep-0357.html).
`fpush` can be connected to any XMPP server as a XMPP component using [XEP-0114](https://xmpp.org/extensions/xep-0114.html).


## Table of contents

- [Features](#features)
- [Usage](#usage)
- [XMPP API](#xmpp-api)
- [Configuration](#configuration)
- [Structure](#structure)
- [Clustering](#clustering)
- [Compilation](#compilation)
- [Compilation Flags](#compilation-flags)
- [Expandability](#expandability)
- [Systemd](#systemd)


<a name="features"></a>
## Features

* Supported push platforms
  * Apple APNS
  * Google FCM
* Multi app / platform support on a single XMPP domain/JID
* Configurable token ratelimiting

<a name="usage"></a>
## Usage

```
./fpush settings.json
```

<a name="xmpp-api"></a>
## XMPP API

This appserver implements [XEP-0357](https://xmpp.org/extensions/xep-0357.html) commands for sending actual push notifications.
`fpush` does not require devices to register for push before usage like other appserver implementations.

`fpush` expects each push IQ to include the push token of the device to be woken up, as well as the info to which push module the token applies.
Hence, push messages can be enabled by a client at their XMPP server as follows:
```XML
<iq type='set' id='x43'>
  <enable xmlns='urn:xmpp:push:0' jid='<pushServerComponentJid>' node='<DevicePushToken>'>
    <x xmlns='jabber:x:data' type='submit'>
      <field var='FORM_TYPE'><value>http://jabber.org/protocol/pubsub#publish-options</value></field>
      <field var='pushModule'><value>appProductionPushModule</value></field>
    </x>
  </enable>
</iq>
```

If the `pushModule` identifier is missing in the publish-options, `fpush` will instead selected the default push module as configured.

<a name="configuration"></a>
### Configuration

```json
{
    "component": {
        "componentHostname": "<ComponentJid>",
        "componentKey": "ARandomComponentKeySetInsideTheXMPPServer",
        "serverHostname": "<XMPP-Server>",
        "serverPort": <XMPP-Server Component Port>
    },
    "pushModules": { // map of configured push modules
        "monalProdiOS": { // push module identifier that is later set inside the push iqs as "pushModule"
            "type": "apple", // push module type
            "is_default_module": true, // Use this push module in case no pushModule was defined in a push iq msg
            "apns": {
                "certFilePath": "<Path to p12 file>",
                "certPassword": "<cert password>",
                "topic": "<app bundle id>"
            },
            "ratelimit": {
                "ratelimitTime": "20s", // minimal duration between two pushes
                "ratelimitCleanupInterval": "300s", // time how long old and unseen tokens should be keept inside the ratelimit cache before removal
                "enabled": true // optionally disable ratelimit. Not advised for apple apns
            }
        },
        "someAndroidApp": {
            "type": "fcm",
            "is_default_module": false,
            "fcm": {
                "fcmSecretPath": "<Path to json from google>"
            }
        },
    },
    "timeoutConfig": {
        "xmppconnectionError": "20s" // time to wait after XMPP component connection failed before reconnecting
    }
}
```

The configuration file consists of three sections.
XMPP component settings (`component`) the push module configurations (`pushModules`) and a timeout config for the xmpp connection (`timeoutConfig`).

### `component`

This section describes all config parameters for the XMPP component connection to the XMPP server handling all S2S connections.

#### `componentHostname`

JID of the pushserver.
Must match the component JID configured on the XMPP server.

#### `componentKey`

The component handshake element as configured on the XMPP server

#### `serverHostname`

Hostname or IP address of the XMPP component endpoint (Usually the XMPP server that handles all S2S connections)

#### `serverPort`

Port of the XMPP component endpoint configured on the XMPP server.

### `pushModules`

Map of all push modules that should be loaded on start.
Each push module can either be for apple APNS or for googles FCM.
Each push module is identified by the key specified in the map.
This identifier is later used by clients to specifiy which of the configured push modules should be selected when an XMPP IQ is received by `fpush`.

Each push module consists of a `type` element.
Currently `apple` and `fcm` are supported.

#### `is_default_module`

If set to true, this push modules is used if an XMPP IQ was received that does not include any push module identifier.
Only one push module can be configured as the default module.

#### `ratelimit`

Ratelimits for push tokens can be configured per push module.

##### `enabled`

Enable or disable ratelimiting for tokens
It is not advised to disable the ratelimiting

##### `ratelimitTime`

Minimal time between pushes for each push token.
If two push IQs are received for the same token within less time, the latter one is queued.
If more push request arrive for a token, while one push request is already queued, all further requests are ignored and an wait IQ is replied until the configured `ratelimitTime` is over.

##### `ratelimitCleanupInterval`

`fpush` caches the last timestamp when an push messages was sent for an token per push module to implement the ratelimit.
This cache is cleaned every 300 seconds.
On each cleaning run, tokens that were send more than `ratelimitCleanupInterval` ago are removed to free up memory space.

#### `apns`

This section describes all apns related push options.

##### `certFilePath``

Path to the p12 certificate that should be used to connect to the APNS API.

##### `certPassword`

Passwort of the p12 certificate.

##### `topic`

Bundle ID of the main app.

#### `fcm`

This section describes all fcm related push options.

##### `fcmSecretPath``

Path to the fcm json file created by google.

### `timeoutConfig`

<a name="structure"></a>
## Structure

Fpush connects to a single XMPP server, that handles all S2S connections, as a XMPP component using an unencrypted connection.
We thus recommend to either place `fpush` on the same system as the XMPP server or securing the connection between the systems using `IPsec` or `wireguard`.

If the XMPP server is unreachable `fpush` will automatically try to reconnect after the configured time.

<a name="clustering"></a>
## Clustering

`fpush` can be clustered by simply running multiple servers on the same XMPP domain.
Using SRV records S2S traffic can be steered between the servers.

`fpush` currently does not exchange ratelimit information betweeen cluster nodes.
Instead each node manages its own independent ratelimit.
Hence, devices that are registered on more than on XMPP server may, receive push notifications via more than one cluster more frequent than expected.

<a name="compilation"></a>
## Compilation

`fpush` is written in Rust. (https://www.rust-lang.org/tools/install)

### Debug build
```bash
cargo build
```

### Release build
```bash
cargo build --release
```

<a name="compilation-flags"></a>
### Compilation Flags

`fpush` supports several features that can be activated while building using the `--feature <Flag1>,<Flag2>` flag.

#### Default features

Currently the following features are enabled by default.
* `random_delay_before_push`
* `enable_apns_support`
* `enable_fcm_support`

The default features can be disabled using `--no-default-features`.

#### Logging

##### release_max_level_warn

Remove all log messages lower than "WARN" from binary to increase speed.

##### release_max_level_info

Remove all log messages lower than "INFO" from binary to increase speed.

#### Push module support

Push modules support for different push vendors can be enabled during compilation.
For improved performance it is adviced to only enable/compile the needed push modules types (e.g. only apns).

##### enable_fcm_support

Enable google fcm support for android devices.

##### enable_apns_support

Enable apple apns support for iOS, iPadOS and macOS devices.

##### enable_demo_support

Enable a simple demo push endpoint used during development.

##### random_delay_before_push

Improve ratelimit acurracy by waiting a random delay before handling each push message.

For each incomming push event fpush spawns a new tokio thread.
Within each thread fpush checks if the supplied token is blocked or if the token is ratelimited.
After a push message was sent for a token, fpush ratelimits the token (if configured) for a configured time to reduce the battery consumption of the remote device.
Due to the multi thread design, two directly consecutive push events for the same token may not be correctly ratelimited.
Hence, adding a random async delay helps to improve the accuracy while keeping memory and cpu consumption low.

<a name="expandability"></a>
### Expandability

Fpush can easily be expanded to support further push platforms by creating a new crate implementing the ```PushTrait```.

<a name="systemd"></a>
### Systemd

We recommend running `fpush` as a seperate user without root permissions.

```
[Unit]
Description=Fpush
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=10
User=fpush
Group=fpush
LimitNOFILE=131072
WorkingDirectory=/opt/fpush/
Environment=RUST_LOG=info
ExecStart=/opt/fpush/fpush settings.json

[Install]
```
