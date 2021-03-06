FROM docker-registry.cs.int/prod/cs-base
ENV DEBIAN_FRONTEND=noninteractive

ARG destEnv="sandbox"
ARG commitSha="unknown"
ARG commitRef="unknown"
ARG ciJobId="unknown"

LABEL maintainer="<devops@collective-sense.com>" \
	com.collective-sense.build-info.sha="${commitSha}" \
	com.collective-sense.build-info.ref="${commitRef}" \
	com.collective-sense.build-info.ci-id="${ciJobId}"

RUN echo "deb http://aptly.cs.int/public xenial $destEnv" >> /etc/apt/sources.list \
	&& printf "Package: * \nPin: release a=xenial, o=aptly.cs.int \nPin-Priority: 1600 \n" > /etc/apt/preferences

RUN groupadd -g 10033 hyena \
	&& groupadd -g 10036 hyena-tmp \
	&& adduser --no-create-home --system --shell /bin/false --gecos 'hyena dedicated user' --uid 10033 --gid 10033 --disabled-password hyena \
	&& usermod -a -G hyena-tmp hyena

COPY *.deb /tmp

RUN apt-get update \
	&& (dpkg -i /tmp/*.deb || true) \
	&& apt-get install -fy --no-install-recommends \
	&& apt-get clean \
	&& rm -rf /var/lib/apt/lists \
	&& rm /tmp/*.deb

COPY files/version.sh /version.sh
RUN ln -s /version.sh /usr/bin/version

USER hyena

CMD /usr/bin/hyena
