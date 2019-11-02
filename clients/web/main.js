angular.module('app', []).constant('_', window._).controller('coreController', function ($scope) {
    $scope.results = [];

    function reload() {
        $.get({ url: 'http://localhost:8080' }).done(rawResult => {
            $scope.$apply(() => {
                $scope.results = {
                    connections: rawResult.connections.sort((a, b) => (b.bytes_downloaded + b.bytes_uploaded) - (a.bytes_downloaded + a.bytes_uploaded)),
                    processes: rawResult.processes
                };
            });
            console.log({ results: $scope.results });
        })
    }

    function timer() {
        setTimeout(() => {
            reload();
            timer();
        }, 1000);
    }

    timer();

    $scope.findProcess = (inode) => {
        const process = _.find($scope.results.processes, process => _.includes(process.inodes, inode));
        return _.get(process, 'executable') || _.get(process, 'command') || _.get(process, 'pid');
    };

    $scope.fs = (bytes) => {
        if (!bytes) return '0 B';
        const sizeIndex = Math.floor(Math.log(bytes) / Math.log(1024)),
            sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

        return (bytes / Math.pow(1024, sizeIndex)).toFixed(2) * 1 + ' ' + sizes[sizeIndex];
    };
});
