import QtQuick 2.12
import QtQuick.Window 2.12
import QtQuick.Layouts 1.3
import QtQuick.Controls 2.12
import QtQuick.Controls.Material 2.12

ApplicationWindow {
    visible: true
    width: 640
    height: 480
    title: "Hello World"

    Material.theme: Material.Light
    Material.accent: Material.Purple

    FontLoader {
    	id: iconFont
        source: "material_icons/MaterialIcons-Regular.ttf"
    }

    function humanReadableBytes(bytes) {
        if (!bytes) return '0 B';
        const sizeIndex = Math.floor(Math.log(bytes) / Math.log(1024)),
            sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

        return (bytes / Math.pow(1024, sizeIndex)).toFixed(2) * 1 + ' ' + sizes[sizeIndex];
    }

    Timer {
        id: timer

        running: true
        triggeredOnStart: true
        interval: 1000
        repeat: true

        property var latestData: {'connections': [], 'processes': []}

        onTriggered: function() {
            request({url: 'http://localhost:8080'})
                .then(result => {
                    timer.latestData = result;
                })
                .catch(() => {
                    console.log('Failed to fetch data');
                })
        }

        function request(options) {
            return new Promise((accept, reject) => {
                var xhr = new XMLHttpRequest();

                xhr.timeout = Math.min(options.timeout, 5000);
                xhr.onreadystatechange = function() {
                    if (xhr.readyState == XMLHttpRequest.DONE) {
                        if (xhr.status == 200) {
                            try {
                                return accept(JSON.parse(xhr.responseText))
                            } catch (e) {
                                return reject(xhr);
                            }
                        }

                        return reject(xhr);
                    }
                }

                xhr.open(options.method || 'GET', options.url, true);

                xhr.send();
            })
        }
    }

    ListView {
        id: connections

        anchors.fill: parent
        anchors.margins: 20
        spacing: 30
        ScrollBar.vertical: ScrollBar {
            parent: connections.parent
            anchors.top: connections.top
            anchors.left: connections.right
            anchors.bottom: connections.bottom
            width: 5
        }

        model: timer.latestData.connections

        delegate: Component {
            id: contactsDelegate

            Rectangle {
                width: parent.width
                height: connectionCard.height
                color: "transparent"

                Pane {
                    id: connectionCard
                    width: parent.width
                    anchors.fill: parent

                    Material.elevation: 6

                    Column {
                        spacing: 10

                        Row {
                            Label { text: modelData.source }
                            Label { text: ' - ' }
                            Label { text: modelData.source }
                        }


                        Label {
                            elide: Text.ElideRight
                            text: getProcessNameByPid(modelData.process_id)
                            function getProcessNameByPid(pid) {
                                const process = timer.latestData.processes[`${pid}`];
                                return process ? process['command'] || process['executable'] || process['pid'] : ' ';
                            }
                        }

                        Row {
                            spacing: 30

                            Row {
                                Label {
                                    font.family: iconFont.name
                                    font.pointSize: 10
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: 'arrow_downward'
                                }
                                Label { text: humanReadableBytes(modelData.bytes_downloaded) }
                            }

                            Row {
                                Label {
                                    font.family: iconFont.name
                                    font.pointSize: 10
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: 'arrow_upward'
                                }
                                Label { text: humanReadableBytes(modelData.bytes_uploaded) }
                            }
                        }
                    }
                }
            }
        }
        focus: true
    }
}
