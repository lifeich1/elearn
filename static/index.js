function cur_test_typeid() {
    return $('select#select-test-type').val();
}

function cur_test_type_name() {
    var id = cur_test_typeid();
    return $('select#select-test-type option[value="' + id + '"]').text();
}

function cur_test_name() {
    return $('select#select-test-name').val();
}


function on_test_type_changed() {
    var id = cur_test_typeid();
    if (id > 0) {
        $('select#select-test-name').load('/card/test_name_option/' + id);
        $('div#editor').html('');
        $('span#type-title-display').text(cur_test_type_name());
        $('span#template-name-display').text('未选中');
    }
}

function on_test_name_changed() {
    var typ = cur_test_typeid();
    var name = cur_test_name();
    if (typ > 0 && name != '未选中') {
        $('div#mainpage').load('/welcome/' + typ + '/' + name, function() {
            $("div#loading-spinner").hide();
            nav_offcanvas_modal.hide();
        });
        $('span#template-name-display').text(name);
    }
}


function do_post_json(url, data, success, complete) {
    if (!complete) {
        complete = function (xml, status) {
            console.log("final", status);
        };
    }
    if (!success) {
        success = function (d) {
            console.log("succ", d);
        };
    }
    $.ajax({
        type: "post",
        url: url,
        dataType : "json",
        contentType : "application/json",
        data: JSON.stringify(data),
        complete: complete,
        success: success,
        error: function(xhr, status, error) {
            show_toast('POST错误', 'status: ' + xhr.status +
                ' ' + status + '<br>error: ' + error);
        },
    });
}

function use_yorn_modal(title, desc, cb) {
    $('div.modal#yes-or-no-modal h5.modal-title').text(title);
    $('div.modal#yes-or-no-modal div.modal-body').html(desc);
    $('div.modal#yes-or-no-modal div.modal-footer button.btn-primary').click(function() {
        yorn_modal.hide();
        if (cb != null) cb();
    });
    yorn_modal.show();
}

var cur_prob_n = null;

function on_begin_test(type, name) {
    use_central_loading_mark();
    $('div#mainpage').load('/exam/' + type + '/' + name + '/10', function() {
        clustering_pick_prob(0);
    });
}

function on_begin_test_sel() {
    var id = cur_test_typeid();
    var name = cur_test_name();
    if (id > 0 && name != '未选中') {
        nav_offcanvas_modal.hide();
        use_yorn_modal('确认', '确认<span class="text-danger">重新开始</span>测试？', function() {
            on_begin_test(id, name);
        });
    }
}

function clustering_pick_prob(n) {
    cur_prob_n = n;
    $('div#mainpage div#test-container div.card').hide();
    var q = $('div#mainpage div#test-container div.card#test-card-' + n);
    if (q.length == 0) {
        clustering_summary();
        return;
    }
    enforce_show(q);
    clustering_set_progress(n);
}

function clustering_set_progress(n, morecls) {
    var q = $('div.progress-bar#test-progressbar');
    if (morecls) {
        q.attr('class', 'progress-bar ' + morecls);
    }
    q.attr('aria-valuenow', String(n));
    let m = parseInt(q.attr('aria-valuemax'));
    q.attr('style', 'width: ' + Math.round(n * 100 / m) + '%;');
    q.text(n + '/' + m);
    return q;
}

function clustering_summary() {
    $('div#mainpage div#test-container div.card').show();
    enforce_show($('button#wa-only-toggle'));
    var n = $('div#test-container div.card[aria-label="AC"]').length;
    clustering_set_progress(n, 'bg-success').text(n + '/' + cur_prob_n + '分');
    save_history();
}

function clustering_toggle_waonly() {
    $('div#test-container div.card[aria-label="AC"]').toggle();
}

function save_history() {
    do_post_json('/save_history', {
        typ: cur_test_typeid(),
        name: cur_test_name(),
        data: $('div#mainpage').html(),
    }, function(d) {
        show_toast('保存测试结果', String(d), d=='保存成功');
    })
}

function show_toast(title, body, short) {
    $('div#liveToast strong').text(title);
    var ts = new Date().toLocaleString();
    $('div#liveToast small').text(ts);
    $('div#liveToast div.toast-body').html(body);
    if (short) {
        setTimeout(function() {
            if ($('div#liveToast small').text() == ts) {
                toast_modal.hide();
            }
        }, 5000);
    }
    toast_modal.show();
}

function enforce_show(q) {
    q.removeAttr('hidden');
    q.show();
}

function on_clustering_choose(itemid, optid, answer) {
    enforce_show($('div#test-container div#option-' + itemid + '-' + optid + ' span.badge'));
    enforce_show($('div#test-container div#option-' + itemid + '-' + answer + ' span.badge'));
    var q = $('div#test-container div.card#test-card-' + itemid);
    if (optid != answer) {
        enforce_show($('div#test-container p#explain-' + itemid));
        q.attr('aria-label', 'WA');
    } else {
        q.attr('aria-label', 'AC');
    }
    $('div#test-container div.card#test-card-' + itemid + ' input').attr('disabled', 'true');
    clustering_pick_prob(itemid + 1);
}

function on_show_history_list(type, name) {
    $('div#mainpage').load('/list_history/' + type + '/' + name);
}

function on_show_history_list_sel() {
    var id = cur_test_typeid();
    var name = cur_test_name();
    if (id > 0 && name != '未选中') {
        on_show_history_list(id, name);
        nav_offcanvas_modal.hide();
    }
}

function on_show_history(tag) {
    var typ = cur_test_typeid();
    var name = cur_test_name();
    $('div#mainpage').load('/history/' + typ + '/' + name + '/' + tag + '.html');
}

function central_loading_mark() {
    return '<div class="d-flex justify-content-center">' +
        '<div class="spinner-border" role="status">' +
        '<span class="visually-hidden">Loading...</span>' +
        '</div>' +
        '</div>';
}

function use_central_loading_mark() {
    $('div#mainpage').html(central_loading_mark());
}


function onResize() {
    $("body").css("padding-top", $("nav.fixed-top").height());
}

var evsrc = null;

var yorn_modal = null;

var toast_modal = null;

var nav_offcanvas_modal = null;

$(function() {
    yorn_modal = new bootstrap.Modal($('div.modal#yes-or-no-modal')[0]);
    toast_modal = new bootstrap.Toast($('div.toast#liveToast')[0]);
    nav_offcanvas_modal = new bootstrap.Offcanvas($('div.offcanvas#offcanvasNavbar')[0]);
    $(window).resize(onResize);
    onResize();
})
