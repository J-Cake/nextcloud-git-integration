FROM rust:trixie AS rust

RUN apt-get update && apt-get install -y openssh-server
RUN mkdir /app

RUN passwd -d root
RUN chsh -s /bin/bash root
RUN printenv | grep -E '^(CARGO_HOME|RUSTUP_HOME|PATH)=' >> /etc/environment
RUN cat /etc/environment | sed 's/^/SetEnv /' >> /etc/ssh/sshd_config

RUN cat <<-'EOF' >> /etc/ssh/sshd_config
	PermitEmptyPasswords yes
	PasswordAuthentication yes
	PermitRootLogin yes
	PermitUserEnvironment yes
	EOF

RUN printf 'PS1=\047\\t \\w\\$ \047\n' > /etc/profile

EXPOSE 1920 22

VOLUME /app
WORKDIR /app

CMD ["/usr/sbin/sshd", "-D"]
