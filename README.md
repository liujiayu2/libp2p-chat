# libp2p-chat
rust版libp2p使用教程

原始资源链接：https://github.com/Lainera/libp2p-chat

启动第一个结点：cargo run 

启动第二个结点并连接到第一个结点：cargo run /ip4/192.168.3.63/tcp/4187

这个时候这两个结点就可以互相通讯了，

启动第三个结点并连接到第一个结点：cargo run /ip4/192.168.3.63/tcp/4187

在第三个窗口输入信息并点击回车按钮，可以看到在前两个窗口收到了信息
