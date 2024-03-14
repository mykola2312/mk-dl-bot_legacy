node {
    def app

    stage('Clone') {
        checkout scm
    }

    stage('Build') {
        withEnv(['DOCKER_BUILDKIT=1']) {
            app = docker.build('mykola2312/mk-dl-bot')
        }
    }

    stage('Push') {
        docker.withRegistry('https://registry.hub.docker.com', 'a2aa5264-dce1-4054-8828-8db95e3c6c3c') {
            app.push('latest')
        }
    }
}