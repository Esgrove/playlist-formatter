#include <QApplication>
#include <QCommandLineOption>
#include <QCommandLineParser>
#include <QtWidgets>

int main(int argc, char* argv[])
{
    QApplication app(argc, argv);
    QApplication::setOrganizationName(QApplication::translate("main", "Esgrove"));
    QApplication::setApplicationName(QApplication::translate("main", "PlaylistTool"));
    QApplication::setApplicationVersion("1.0.0");

    QCommandLineParser parser;
    parser.setApplicationDescription(QApplication::translate("main", "Playlist Tool"));
    parser.addHelpOption();
    parser.addVersionOption();
    parser.process(app);

    QWidget window;
    window.resize(800, 600);
    window.show();
    window.setWindowTitle(QApplication::translate("toplevel", "Esgrove's Playlist Tool"));

    return app.exec();
}
