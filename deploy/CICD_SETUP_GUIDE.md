# CI/CD 局域网服务器配置指南 (Ubuntu + k3s)

为了让 GitHub Actions 能够自动部署到您的局域网服务器 (`10.0.0.101`)，我们需要在服务器上配置 **Self-hosted Runner**。

## 1. 安装 GitHub Self-hosted Runner

1.  登录您的 GitHub 仓库。
2.  依次点击 **Settings** -> **Actions** -> **Runners**。
3.  点击 **New self-hosted runner**。
4.  选择 **Linux** -> **x64**。
5.  按照页面上的指令（Download & Configure）在您的 **Ubuntu 服务器**上运行命令。
    -   *提示*: 在执行 `./config.sh` 时，您可以给 runner 起个名字，例如 `ubuntu-k3s-node`。
6.  配置完成后，运行 `./run.sh` 启动 runner。
    -   *建议*: 生产环境建议将其安装为服务：`sudo ./svc.sh install` 和 `sudo ./svc.sh start`。

## 2. 准备 Kubernetes 命名空间与密钥

在服务器上运行以下命令，为服务准备必要的 Secret（请根据实际情况替换值）：

```bash
# 创建命名空间
kubectl create namespace cuba-system

# 创建数据库密钥
kubectl create secret generic db-secrets \
  --from-literal=database-url="postgresql://postgres:postgres@10.0.0.101:5432/cuba" \
  -n cuba-system

# 创建 JWT 密钥
kubectl create secret generic jwt-secrets \
  --from-literal=secret="your-super-secret-key" \
  -n cuba-system
```

## 3. 验证权限

确保运行 runner 的用户有权限执行以下命令：
-   `docker build`
-   `kubectl`
-   `sudo k3s ctr` (用于导入镜像到 k3s)

如果遇到权限问题，请将用户加入 `docker` 组：
```bash
sudo usermod -aG docker $USER
newgrp docker
```

## 4. 自动部署流程

1.  您在 Mac 上修改代码。
2.  `git commit` 并 `git push` 到分支。
3.  GitHub CI 运行测试 (使用 Rust 1.93)。
4.  当您将代码合并到 `init` 分支时，Ubuntu 服务器上的 Runner 会自动：
    -   下拉代码。
    -   构建 Docker 镜像。
    -   将镜像同步到 k3s。
    -   执行 `kubectl apply` 更新服务。
