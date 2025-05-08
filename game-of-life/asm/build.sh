#!/bin/sh

set -ex

SPECTOOL="${SPECTOOL:-spectool}"
$SPECTOOL prepare ./game-of-life.psm > game-of-life1.json
sed 's/address.*/address\": 131072,/' game-of-life1.json > game-of-life.json
rm game-of-life1.json
