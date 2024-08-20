FROM alpine:3.18

# Arguement for Password
ARG PASSWORD=root

ARG LISTEN_PORT=3420

#Installing the openssh and bash package, removing the apk cache
RUN apk --update add --no-cache zsh-syntax-highlighting zsh-autosuggestions openssh-server htop tcpdump socat neofetch zsh curl git bat fd ripgrep iptables vim \
  && sed -i s/#PermitRootLogin.*/PermitRootLogin\ yes/ /etc/ssh/sshd_config \
  && sed -i s/#PermitTTY.*/PermitTTY\ yes/ /etc/ssh/sshd_config \
  && sed -i s/#PrintMotd.*/PrintMotd\ yes/ /etc/ssh/sshd_config \
  && sed -i s/#PermitTunnel.*/PermitTunnel\ yes/ /etc/ssh/sshd_config \
  && sed -i s/#AllowAgentForwarding.*/AllowAgentForwarding\ yes/ /etc/ssh/sshd_config \ 
  && sed -i s/AllowTcpForwarding.*/AllowTcpForwarding\ yes/ /etc/ssh/sshd_config \ 
  && sed -i s/GatewayPorts.*/GatewayPorts\ yes/ /etc/ssh/sshd_config \
  && echo "root:${PASSWORD}" | chpasswd \
  && rm -rf /var/cache/apk/*


RUN sh -c "$(wget https://raw.github.com/robbyrussell/oh-my-zsh/master/tools/install.sh -O -)"

RUN echo "source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh" >> ~/.zshrc \
  && echo "source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh" >> ~/.zshrc

RUN echo "export PATH=/sbin/:$PATH" >> /root/.zshrc
RUN echo "export TERM=xterm" >> /root/.zshrc

# Defining the Port 22 for service
RUN sed -ie "s/#Port 22/Port ${LISTEN_PORT}/g" /etc/ssh/sshd_config
RUN /usr/bin/ssh-keygen -A
RUN ssh-keygen -t rsa -b 4096 -f  /etc/ssh/ssh_host_key


