#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <unistd.h>

static void record_attempt(void) {
    const char *path = getenv("RRC_NETWORK_MARKER");
    if (path == NULL) return;
    int fd = open(path, O_WRONLY | O_CREAT | O_APPEND, 0600);
    if (fd >= 0) {
        const char message[] = "internet socket attempted\n";
        (void)write(fd, message, sizeof(message) - 1);
        (void)close(fd);
    }
}

int socket(int domain, int type, int protocol) {
    static int (*real_socket)(int, int, int);
    if (domain == AF_INET || domain == AF_INET6) {
        record_attempt();
        errno = EPERM;
        return -1;
    }
    if (real_socket == NULL) real_socket = dlsym(RTLD_NEXT, "socket");
    return real_socket(domain, type, protocol);
}

int connect(int socket_fd, const struct sockaddr *address, socklen_t length) {
    static int (*real_connect)(int, const struct sockaddr *, socklen_t);
    if (address != NULL && (address->sa_family == AF_INET || address->sa_family == AF_INET6)) {
        record_attempt();
        errno = EPERM;
        return -1;
    }
    if (real_connect == NULL) real_connect = dlsym(RTLD_NEXT, "connect");
    return real_connect(socket_fd, address, length);
}
