pipeline {
    agent {
        docker { image 'rust:latest' }
    }
    stages {
        stage('Test') {
            steps {
                sh 'cargo test'
            }
        }
        stage('Build') {
            steps {
                sh 'cargo build --release'
            }
        }
    }
	post {
        always {
            archiveArtifacts artifacts: 'target/release/mapmaster', fingerprint: true, onlyIfSuccessful: true
        }
    }
}
