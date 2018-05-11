#!/bin/bash

set -ex

BUILD_DOCKER_IMAGE=true
DOCKERFILES_TO_BUILD="Dockerfile"
DOCKER_IMAGE_BUILD_DIR="dockerization"

destEnv="${destEnv:-dev}"

function retry() {
  count=$1
  slp=$2
  cmd=$3

  ( for i in $(seq 0 $count); do
    [ $i -gt 0 ] && echo "---- Retrying $i time ----"; $cmd && break || [ $i -lt $count ] && echo "---- FAILURE, waiting $slp secs ----" && sleep $slp || exit;
  done ) || return 1
}

cd build

DEB_PACKAGES=$(ls *.deb)
debVer=

for deb in ${DEB_PACKAGES}; do
  APTLY_SERVER="aptly.cs.int"
  APTLY_HTTP="http://${APTLY_SERVER}:8080"

  retry 5 15 "curl -X POST -F file=@${deb} ${APTLY_HTTP}/api/files/${deb%_amd64.*}"
  retry 5 15 "curl -X POST ${APTLY_HTTP}/api/repos/${destEnv}/file/${deb%_amd64.*}"

  ssh -o StrictHostKeyChecking=no -tt aptly@${APTLY_SERVER}

  # even if we build two packages in one build they *should* have the same version so we can do it like this:
  debVer="$(dpkg-deb -f ${deb} Version)"
done

#echo version="${VERSION}" > ${SOURCE}/env.properties

LOCAL_DOCKER_IMAGE="cs/hyena"
PORTUS_DOCKER_IMAGE="cs-hyena"

cd ../${DOCKER_IMAGE_BUILD_DIR}
for dockerfile in ${DOCKERFILES_TO_BUILD}; do
echo "Building ${DOCKER_IMAGE_BUILD_DIR}/${dockerfile}..."


sudo docker build --file ${dockerfile} --build-arg destEnv=${destEnv} --build-arg debVer=${debVer} --no-cache -t cs/${LOCAL_DOCKER_IMAGE} .
sudo docker tag cs/${LOCAL_DOCKER_IMAGE} portus.cs.int:5000/${destEnv}/${PORTUS_DOCKER_IMAGE}
retry 5 15 "sudo docker push portus.cs.int:5000/${destEnv}/${PORTUS_DOCKER_IMAGE}"

done
