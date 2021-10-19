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
        $('div#editor').load('/editor/' + typ + '/' + name, function() {
            $("div#loading-spinner").hide();
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
    });
}


function on_new_test() {
    if (cur_test_typeid() == 0) {
        use_yorn_modal('警告', '<span class="text-danger">需要先选择测试类型</span>');
        return;
    }
    use_yorn_modal('新建测试',
        '<div class="input-group">' +
        '<span class="input-group-text">测试名称</span>' +
        '<input id="input-new-test-name" type="text" class="form-control" placeholder="名称">' +
        '</div>', function() {
            var name = $('input#input-new-test-name').val();
            $('select#select-test-name').append('<option>' + name + '</option>');
            $('select#select-test-name').val(name);
            on_test_name_changed();
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

function on_clustering_input_keydown(event, id) {
    if (event.keyCode == 13) {
        on_clustering_new_col(id);
    }
}

function on_clustering_new_col(id) {
    var input = $('div#data-container div.card#data-card-' + id + ' input.form-control');
    var data = input.val();
    input.val('');
    $('div#data-container div.card#data-card-' + id + ' div.d-flex').append('<div class="p-2 ms-1" aria-label="' + data + '">' +
        '<span>' + data + '</span>' +
        '<button type="button" class="btn-close" aria-label="Delete" onclick="clustering_remove_item(\'' + data + '\')"></button>' +
        '</div>');
}

function clustering_new_row() {
    var cid = $('div#data-container div.card:last-child').attr('id');
    var lid = '0';
    if (cid != undefined) {
        lid = cid.replace('data-card-', '');
    }
    var id = parseInt(lid) + 1;
    $('div#data-container').append('<div class="card m-3 shadow" id="data-card-' + id + '">' +
        '<div class="card-body">' +
            '<div class="input-group">' +
                '<input type="text" class="form-control" placeholder="Item">' +
                '<button class="btn btn-outline-primary" type="button" onclick="on_clustering_new_col(\'' + id + '\')">' +
                    '添加' +
                '</button>' +
            '</div>' +
            '<div class="d-flex">' +
            '</div>' +
        '</div>' +
    '</div>');
    $('div#data-container div.card#data-card-' + id + ' input').keydown(function(e) {
        if (e.keyCode == 13) {
            on_clustering_new_col(id);
        }
    })
}

function clustering_remove_item(data) {
    $('div#data-container div.card div.d-flex div[aria-label="' + data + '"]').remove();
}

function clustering_current_data() {
    var q = $('div#data-container div.card');
    var result = [];
    for (i = 0; i < q.length; ++i) {
        var cid = q[i].getAttribute('id');
        var id = parseInt(cid.replace('data-card-', ''));
        var p = $('div#data-container div.card#' + cid + ' div.d-flex div[aria-label] span');
        for (j = 0; j < p.length; ++j) {
            result.push({
                clsid: id,
                data: p[j].getInnerHTML(),
            });
        }
    }
    return result;
}

function clustering_submit() {
    $("div#loading-spinner").show();
    do_post_json('/submit/1/' + cur_test_name(),
        {
            data: clustering_current_data(),
        }, function(d) {
            var desc = String(d) + ' - ' + new Date().toLocaleString();
            console.log(desc);
            $('span#clustering-last-submit-time').text(desc);
        }, function() {
            console.log('fin');
            $("div#loading-spinner").hide();
        }
    );
}


function onResize() {
    $("body").css("padding-top", $("nav.fixed-top").height());
}

var evsrc = null;

var yorn_modal = null;

$(function() {
    yorn_modal = new bootstrap.Modal($('div.modal#yes-or-no-modal')[0]);
    $(window).resize(onResize);
    onResize();
})
