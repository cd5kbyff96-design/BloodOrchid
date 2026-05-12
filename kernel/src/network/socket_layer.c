/**
 * kernel/src/network/socket_layer.c
 */

#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <string.h>

int socket_create_server(int port) {
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0) return -1;
    
    int opt = 1;
    setsockopt(sock, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));
    
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(port);
    
    if (bind(sock, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        close(sock);
        return -1;
    }
    
    if (listen(sock, 10) < 0) {
        close(sock);
        return -1;
    }
    
    return sock;
}

int socket_accept_connection(int server_sock) {
    struct sockaddr_in client_addr;
    socklen_t client_len = sizeof(client_addr);
    return accept(server_sock, (struct sockaddr*)&client_addr, &client_len);
}

int socket_send_data(int sock, const void* data, size_t size) {
    return send(sock, data, size, 0);
}

int socket_receive_data(int sock, void* buffer, size_t max_size) {
    return recv(sock, buffer, max_size, 0);
}

void socket_close(int sock) {
    close(sock);
}